#include "timeline.h"
#include "download.h"
#include "downloadworker.h"

#include "downloadtask.h"

#include <iostream>
#include <sstream>
#include <fstream>

#include <QDateTime>
#include <QTimeZone>
#include <QLabel>
#include <QScrollArea>
#include <QDateTime>
#include <QTimeZone>
#include <QVBoxLayout>
#include <QListView>
#include <QStandardItemModel>
#include <QStandardItem>
#include <QAbstractItemView>
#include <QAbstractScrollArea>
#include <QDebug>
#include <QThreadPool>

#include "SimpleIni.h"
#include "../widgets/richtextdelegate.h"

#include "../../config.h"

#include <algorithm>

#include <random>

namespace twtgui
{
    unsigned int wordToUint(const std::string &word)
    {
        constexpr unsigned int FNV_prime = 16777619u;
        unsigned int hash = 2166136261u; // FNV offset basis

        for (unsigned char c : word)
            hash = (hash ^ c) * FNV_prime;

        return hash;
    }

    // utilites for making text more readable

    float calculateRelativeLuminance(int r, int g, int b)
    {
        // convert color to sRGB

        float RsRGB = r / 255.0f;
        float GsRGB = g / 255.0f;
        float BsRGB = b / 255.0f;

        auto evalChannel = [](const float channel) -> float
        {
            if (channel <= 0.03928)
            {
                return channel / 12.92f;
            }
            else
            {
                float base = (channel + 0.055f) / 1.055f;
                return pow(base, 2.4);
            }
        };

        RsRGB = evalChannel(RsRGB);
        GsRGB = evalChannel(GsRGB);
        BsRGB = evalChannel(BsRGB);

        return 0.2126f * RsRGB + 0.7152f * GsRGB + 0.0722f * BsRGB;
    }

    std::string generateColorFromWord(std::string word)
    {
        uint hash = wordToUint(word);

        int r = (hash * 23141) % 255;
        int g = (hash * 93625) % 255;
        int b = (hash * 67410) % 255;

        QPalette pal;
        QColor color = pal.color(QPalette::Window);

        float L1 = calculateRelativeLuminance(r, g, b);                                  // text
        float L2 = calculateRelativeLuminance(color.red(), color.green(), color.blue()); // bg

        float contrast = (L2 > L1) ? (L2 + 0.05f) / (L1 + 0.05f) : (L1 + 0.05f) / (L2 + 0.05f);
        float adjust = 1.0f - (contrast - 1) / 20.0f;

        if (L2 > L1)
        {
            // decrease brightness
            r = static_cast<uint8_t>(std::max(0, r - static_cast<uint8_t>(128 * adjust)));
            g = static_cast<uint8_t>(std::max(0, g - static_cast<uint8_t>(128 * adjust)));
            b = static_cast<uint8_t>(std::max(0, b - static_cast<uint8_t>(128 * adjust)));
        }
        else
        {
            // increase brightness
            r = static_cast<uint8_t>(std::min(255, r + static_cast<uint8_t>(128 * adjust)));
            g = static_cast<uint8_t>(std::min(255, g + static_cast<uint8_t>(128 * adjust)));
            b = static_cast<uint8_t>(std::min(255, b + static_cast<uint8_t>(128 * adjust)));
        }

        return "rgb(" + std::to_string(r) + "," + std::to_string(g) + "," + std::to_string(b) + ");";
    }

    twtgui::Timeline::Timeline(QWidget *parent)
        : QWidget(parent)
    {
        this->config.LoadFile("config.ini");
        mainLayout = new QVBoxLayout(this);

        // refresh button
        refreshButton = new QPushButton("Refresh", this);
        connect(refreshButton, &QPushButton::clicked, this, &Timeline::handleButtonClick);

        connect(this, &Timeline::allTweetsReady, this, &Timeline::updateTweetsView, Qt::QueuedConnection);

        // list view for tweets
        tweetsView = new QListView(this);
        tweetsModel = new QStandardItemModel(this);
        tweetsView->setModel(tweetsModel);
        tweetsView->setUniformItemSizes(false);
        tweetsView->setWordWrap(true);
        tweetsView->setEditTriggers(QAbstractItemView::NoEditTriggers);
        tweetsView->setSelectionMode(QAbstractItemView::NoSelection);
        tweetsView->setSizeAdjustPolicy(QAbstractScrollArea::AdjustToContents);
        tweetsView->setViewMode(QListView::ListMode);
        tweetsView->setMinimumHeight(512);
        tweetsView->setMinimumWidth(512);
        tweetsView->setItemDelegate(new RichTextDelegate(this));
        tweetsView->viewport()->installEventFilter(this);

        // status label
        statusLabel = new QLabel(this);
        mainLayout->addWidget(tweetsView);
        mainLayout->addWidget(refreshButton);
        mainLayout->addWidget(statusLabel);
        setLayout(mainLayout);

        refreshTimeline(); // initial load
    }

    void twtgui::Timeline::stopWorkers()
    {
        std::vector<DownloadTask *> local;

        {
            std::lock_guard<std::mutex> lk(workerMutex);
            local = tasks;
            tasks.clear();
            pendingWorkers = 0;
        }

        for (auto *task : local)
        {
            if (task)
            {
                QMetaObject::invokeMethod(task, "cancel", Qt::QueuedConnection);
                task->disconnect(this);
            }
        }
    }

    void twtgui::Timeline::onWorkerTweet(const QString &timestamp, const QString &content, const QString &source)
    {
        qDebug() << "Timeline::onWorkerTweet:" << timestamp << "from" << source;
        QDateTime dt = QDateTime::fromString(timestamp, Qt::ISODate);
        {
            std::lock_guard<std::mutex> lk(workerMutex);
            Tweet t;
            t.timestamp = timestamp.toStdString();
            t.content = content.toStdString();
            t.source = source.toStdString();
            collectedTweets.push_back(t);
        }
        return;
    }

    void twtgui::Timeline::onWorkerStatus(const QString &statusMsg)
    {
        statusLabel->setText(statusMsg);
        return;
    }

    void twtgui::Timeline::onWorkerFinished()
    {
        auto *task = qobject_cast<DownloadTask *>(sender());

        if (task)
        {
            // remove task from vector
            std::lock_guard<std::mutex> lk(workerMutex);
            auto it = std::find(tasks.begin(), tasks.end(), task);
            if (it != tasks.end())
                tasks.erase(it);
        }

        // If there are no more pending workers, emit signal to update UI
        {
            std::lock_guard<std::mutex> lk(workerMutex);
            if (!tasks.empty())
                return;
        }

        emit allTweetsReady();
    }

    void Timeline::updateTweetsView()
    {
        qDebug() << "Timeline::onWorkerFinished rebuilding view";

        qDebug() << "Timeline::onWorkerFinished sorting";

        std::vector<Tweet> local;
        {
            std::lock_guard<std::mutex> lk(workerMutex);
            local.swap(collectedTweets); // take ownership of all tweets safely
        }

        std::sort(local.begin(), local.end(), [](const Tweet &a, const Tweet &b)
                  { return QDateTime::fromString(QString::fromStdString(a.timestamp), Qt::ISODate) < QDateTime::fromString(QString::fromStdString(b.timestamp), Qt::ISODate); });

        for (const auto &tweet : local)
            addTweet(tweet.timestamp, tweet.content, tweet.source);

        statusLabel->setText("All feeds loaded");
    }

    void twtgui::Timeline::addTweet(std::string timestamp, std::string content, std::string source)
    {
        // if a source wasn't provided, fall back to configured nick
        if (source.empty())
        {
            source = twtgui::GlobalConfig::config.GetValue("settings", "nick", "unknown");
        }

        std::string color = std::string(twtgui::GlobalConfig::config.GetValue("settings", "colored_names", "0")) == "1" ? "color: " + generateColorFromWord(source) : "";
        QDateTime dt = QDateTime::fromString(QString::fromStdString(timestamp), Qt::ISODate);

        QString text = dt.toString("MM-dd-yyyy hh:mm AP") + " " + "<span style='" + QString::fromStdString(color) + "'><b>" + QString::fromStdString(source) + "</b></span>: " + QString::fromStdString(content);
        QStandardItem *item = new QStandardItem();
        item->setData(text, Qt::DisplayRole);
        item->setEditable(false);
        tweetsModel->insertRow(0, item);
    }

    void twtgui::Timeline::refreshTimeline()
    {
        // stop any in-flight workers
        stopWorkers();

        tweetsModel->clear();
        collectedTweets.clear();

        std::string username = twtgui::GlobalConfig::config.GetValue("settings", "nick", "unknown");

        // add tweets from twtxt file (synchronous; local file reads are cheap)
        std::ifstream file(twtgui::GlobalConfig::config.GetValue("settings", "twtxt", ""));
        if (!file.is_open())
        {
            tweetsModel->appendRow(new QStandardItem("Could not open twtxt file."));
        }
        else
        {
            std::string line;
            while (std::getline(file, line))
            {
                size_t tab = line.find('\t');
                if (tab == std::string::npos)
                    continue;

                std::string timestamp = line.substr(0, tab);
                std::string value = line.substr(tab + 1);

                Tweet t;
                t.timestamp = timestamp;
                t.content = value;
                t.source = username;
                collectedTweets.push_back(t);
            }
            file.close();
        }

        // spawn worker for each following feed (downloads/parsing happen in background)
        CSimpleIniA::TNamesDepend keys;
        twtgui::GlobalConfig::config.GetAllKeys("following", keys);
        CSimpleIniA::TNamesDepend::const_iterator it;

        if (keys.empty())
        {
            // display the existing collected (local) tweets if the user hasn't followed anyone
            std::sort(collectedTweets.begin(), collectedTweets.end(), [](const auto &a, const auto &b)
                      {
                        QDateTime ad = QDateTime::fromString(QString::fromStdString(a.timestamp), Qt::ISODate);
                        QDateTime bd = QDateTime::fromString(QString::fromStdString(b.timestamp), Qt::ISODate);
                        return ad < bd; });
            for (const auto &tweet : collectedTweets)
            {
                addTweet(tweet.timestamp, tweet.content, tweet.source);
            }

            return;
        }

        for (it = keys.begin(); it != keys.end(); ++it)
        {
            const char *key = it->pItem;
            const char *value = twtgui::GlobalConfig::config.GetValue("following", key, nullptr);
            if (value == nullptr)
                continue;

            // derive a source name: prefer the config key if present, otherwise derive host from URL
            std::string sourceName = key ? std::string(key) : std::string();
            if (sourceName.empty())
            {
                std::istringstream ssUrl(value);
                std::string part;
                std::vector<std::string> parts;
                while (std::getline(ssUrl, part, '/'))
                    parts.push_back(part);
                if (parts.size() > 2)
                    sourceName = parts[2];
            }

            QThreadPool::globalInstance()->setMaxThreadCount(4);

            auto *task = new twtgui::DownloadTask(
                QString::fromStdString(value),
                QString::fromStdString(sourceName));

            connect(task, &DownloadTask::status,
                    this, &Timeline::onWorkerStatus, Qt::QueuedConnection);
            connect(task, &DownloadTask::error, this, [this](const QString &err)
                    { statusLabel->setText(err); }, Qt::QueuedConnection);
            connect(task, &DownloadTask::tweetReady,
                    this, &Timeline::onWorkerTweet, Qt::QueuedConnection);
            connect(task, &DownloadTask::finished, this, [this, task]()
                    {
                        {
                            std::lock_guard<std::mutex> lk(workerMutex);
                            tasks.erase(std::remove(tasks.begin(), tasks.end(), task), tasks.end());
                            pendingWorkers = std::max(0, pendingWorkers - 1);
                        }

                        onWorkerFinished(); });

            {
                std::lock_guard<std::mutex> lk(workerMutex);
                tasks.push_back(task);
                pendingWorkers++;
                qDebug() << "pendingWorkers now" << pendingWorkers;
            }

            QThreadPool::globalInstance()->start(task);
        }
    }

    void twtgui::Timeline::handleButtonClick()
    {
        qDebug() << "Refresh button clicked!";
        refreshTimeline();
    }

    twtgui::Timeline::~Timeline() {}

    // for the random colors. turns a word/string into a seed (unsigned int)

} // namespace twtgui