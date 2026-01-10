#include "view_feed.h"

#include <iostream>
#include <sstream>
#include <fstream>

#include <QScrollArea>
#include <QDateTime>
#include <QTimeZone>
#include <QVBoxLayout>
#include <QListView>
#include <QStandardItemModel>
#include <QAbstractItemView>
#include <QAbstractScrollArea>
#include <QDebug>

#include "download.h"
#include "SimpleIni.h"
#include "downloadworker.h"
#include "../widgets/richtextdelegate.h"

#include "../../config.h"

#include <algorithm>

namespace twtgui
{
    twtgui::ViewFeed::ViewFeed(QWidget *parent)
        : QWidget(parent)
    {

        mainLayout = new QVBoxLayout(this);

        // refresh button
        refreshButton = new QPushButton("Refresh", this);
        connect(refreshButton, &QPushButton::clicked, this, &ViewFeed::handleButtonClick);

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

        // initial load: try config's twturl
        std::string initialUrl = twtgui::GlobalConfig::config.GetValue("settings", "twturl", "");
        if (!initialUrl.empty())
        {
            refreshTimeline(initialUrl);
        }
    }

    void twtgui::ViewFeed::onWorkerTweet(const QString &timestamp, const QString &content, const QString &source)
    {
        QDateTime dt = QDateTime::fromString(timestamp, Qt::ISODate);
        {
            std::lock_guard<std::mutex> lk(workerMutex);
            collectedTweets.emplace_back(dt, content.toStdString(), source.toStdString());
        }

        // // Add an immediate item for responsiveness; we'll rebuild sorted view on finished
        // QString text = dt.toString("MM-dd-yyyy hh:mm AP") + " " + "<b>" + source + "</b>: " + content;
        // QStandardItem *item = new QStandardItem();
        // item->setData(text, Qt::DisplayRole);
        // item->setEditable(false);
        // tweetsModel->insertRow(0, item);
    }

    void twtgui::ViewFeed::onWorkerStatus(const QString &statusMsg)
    {
        statusLabel->setText(statusMsg);
    }

    void twtgui::ViewFeed::onWorkerFinished()
    {
        // rebuild sorted view from collectedTweets
        std::vector<std::tuple<QDateTime, std::string, std::string>> local;
        {
            std::lock_guard<std::mutex> lk(workerMutex);
            local = collectedTweets;
            collectedTweets.clear();
        }

        // clear and rebuild in chronological order
        // tweetsModel->clear();
        std::sort(local.begin(), local.end(), [](const auto &a, const auto &b)
                  { return std::get<0>(a) < std::get<0>(b); });
        for (const auto &t : local)
        {
            QString text = std::get<0>(t).toString("MM-dd-yyyy hh:mm AP") + " " + "<b>" + QString::fromStdString(std::get<2>(t)) + "</b>: " + QString::fromStdString(std::get<1>(t));
            QStandardItem *item = new QStandardItem();
            item->setData(text, Qt::DisplayRole);
            item->setEditable(false);
            tweetsModel->insertRow(0, item);
        }

        statusLabel->setText("Successfully loaded tweets");

        // cleanup worker/thread
        if (workerObj)
        {
            workerObj->deleteLater();
            workerObj = nullptr;
        }
        if (workerThread)
        {
            // thread will quit when worker destroyed (we connected destroyed->quit)
            workerThread = nullptr;
        }
    }

    void twtgui::ViewFeed::stopWorker()
    {
        if (workerObj && workerThread)
        {
            QMetaObject::invokeMethod(workerObj, "cancel", Qt::QueuedConnection);
            workerObj = nullptr;
            workerThread = nullptr;
        }
    }

    void twtgui::ViewFeed::refreshTimeline(std::string url)
    {
        stopWorker();

        if (url.empty())
        {
            tweetsModel->appendRow(new QStandardItem("No URL to refresh."));
            statusLabel->setText("No URL to refresh.");
            return;
        }

        // store last used URL so Refresh button can re-download it
        lastUrl = url;

        statusLabel->setText(QString("Downloading from %1 ...").arg(QString::fromStdString(url)));

        // determine username: prefer the name (key) in the "following" section whose value equals
        // this URL; if not found, fall back to using the host part of the URL
        auto hostFromUrl = [](const std::string &u) -> std::string
        {
            if (u.empty())
                return "";
            size_t pos = u.find("://");
            size_t start = (pos == std::string::npos) ? 0 : pos + 3;
            size_t end = u.find('/', start);
            return u.substr(start, end == std::string::npos ? std::string::npos : end - start);
        };

        std::string username = "";
        // search existing following keys for a matching value
        CSimpleIniA::TNamesDepend keys;
        twtgui::GlobalConfig::config.GetAllKeys("following", keys);
        for (CSimpleIniA::TNamesDepend::const_iterator it = keys.begin(); it != keys.end(); ++it)
        {
            const char *key = it->pItem;
            const char *value = twtgui::GlobalConfig::config.GetValue("following", key, nullptr);
            if (value != nullptr && url == value)
            {
                username = key;
                break;
            }
        }

        // fallback to host part if no key found
        if (username.empty())
        {
            username = hostFromUrl(url);
        }

        tweetsModel->clear();

        // start worker in background
        QThread *thread = new QThread(this);
        auto *worker = new twtgui::DownloadWorker();
        worker->moveToThread(thread);

        workerObj = worker;
        workerThread = thread;

        connect(thread, &QThread::started, [worker, url, username]()
                { QMetaObject::invokeMethod(worker, "start", Qt::QueuedConnection, Q_ARG(QString, QString::fromStdString(url)), Q_ARG(QString, QString::fromStdString(username))); });

        connect(worker, &twtgui::DownloadWorker::tweetReady, this, &ViewFeed::onWorkerTweet);
        connect(worker, &twtgui::DownloadWorker::status, this, &ViewFeed::onWorkerStatus);
        connect(worker, &twtgui::DownloadWorker::error, this, [this](const QString &err)
                { statusLabel->setText(err); });
        connect(worker, &twtgui::DownloadWorker::finished, this, &ViewFeed::onWorkerFinished);

        connect(worker, &QObject::destroyed, thread, &QThread::quit);
        connect(thread, &QThread::finished, thread, &QObject::deleteLater);

        thread->start();
    }

    void twtgui::ViewFeed::handleButtonClick()
    {
        qDebug() << "Refresh button clicked!";

        if (!lastUrl.empty())
        {
            refreshTimeline(lastUrl);
            return;
        }

        // fallback to config
        std::string configUrl = twtgui::GlobalConfig::config.GetValue("twtxt", "twturl", "");
        if (!configUrl.empty())
        {
            refreshTimeline(configUrl);
            return;
        }

        tweetsModel->appendRow(new QStandardItem("No URL configured to refresh."));
    }

    twtgui::ViewFeed::~ViewFeed() {}

} // namespace twtgui