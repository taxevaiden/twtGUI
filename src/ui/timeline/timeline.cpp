#include "timeline.h"

#include <iostream>
#include <fstream>

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

#include "SimpleIni.h"
#include "../widgets/richtextdelegate.h"

namespace twtgui {

twtgui::Timeline::Timeline(QWidget *parent, std::string configFile)
    : QWidget(parent), configFile(configFile)
{
    mainLayout = new QVBoxLayout(this);

    // refresh button
    refreshButton = new QPushButton("Refresh", this);
    connect(refreshButton, &QPushButton::clicked, this, &Timeline::handleButtonClick);

    // list view for tweets
    tweetsView = new QListView(this);
    tweetsModel = new QStandardItemModel(this);
    tweetsView->setModel(tweetsModel);
    tweetsView->setUniformItemSizes(false);
    tweetsView->setWordWrap(true);
    tweetsView->setEditTriggers(QAbstractItemView::NoEditTriggers);
    tweetsView->setSelectionMode(QAbstractItemView::NoSelection);
    tweetsView->setSizeAdjustPolicy(QAbstractScrollArea::AdjustToContents);
    tweetsView->setMinimumHeight(512);
    tweetsView->setMinimumWidth(512);

    // render items as rich text (HTML)
    tweetsView->setItemDelegate(new RichTextDelegate(this));

    mainLayout->addWidget(tweetsView);
    mainLayout->addWidget(refreshButton);
    setLayout(mainLayout);

    refreshTimeline(); // initial load
}

void twtgui::Timeline::addTweet(std::string timestamp, std::string content)
{
    CSimpleIniA config;
    config.LoadFile(configFile.c_str());
    std::string username = config.GetValue("twtxt", "nick", "unknown");

    QDateTime dt = QDateTime::fromString(QString::fromStdString(timestamp), Qt::ISODate);

    QString text = dt.toString("MM-dd-yyyy hh:mm AP") + " " + "<b>" + QString::fromStdString(username) + "</b>: " + QString::fromStdString(content);
    QStandardItem *item = new QStandardItem();
    item->setData(text, Qt::DisplayRole);
    item->setEditable(false);
    tweetsModel->insertRow(0, item);
} 

void twtgui::Timeline::refreshTimeline()
{
    tweetsModel->clear();

    CSimpleIniA config;
    config.LoadFile(configFile.c_str());
    std::string username = config.GetValue("twtxt", "nick", "unknown");

    std::ifstream file(config.GetValue("twtxt", "twtfile", ""));
    if (!file.is_open())
    {
        tweetsModel->appendRow(new QStandardItem("Could not open twtxt file."));
        return;
    }

    std::string line;
    std::vector<std::pair<QDateTime, std::string>> tweets;

    while (std::getline(file, line))
    {
        size_t tab = line.find('\t');
        if (tab == std::string::npos)
            continue;

        std::string timestamp = line.substr(0, tab);
        std::string value = line.substr(tab + 1);

        QDateTime dt = QDateTime::fromString(QString::fromStdString(timestamp), Qt::ISODate);
        tweets.emplace_back(dt, value);
    }
    file.close();

    // sort tweets by datetime ascending (oldest first)
    std::sort(tweets.begin(), tweets.end(), [](const auto &a, const auto &b)
              { return a.first < b.first; });

    for (const auto &tweet : tweets)
    {
        QString text = tweet.first.toString("MM-dd-yyyy hh:mm AP") + " " + "<b>" + QString::fromStdString(username) + "</b>: " + QString::fromStdString(tweet.second);
        QStandardItem *item = new QStandardItem();
        item->setData(text, Qt::DisplayRole);
        item->setEditable(false);
        tweetsModel->insertRow(0, item);
    }
}

void twtgui::Timeline::handleButtonClick()
{
    qDebug() << "Refresh button clicked!";
    refreshTimeline();
}

twtgui::Timeline::~Timeline() {}

} // namespace twtgui