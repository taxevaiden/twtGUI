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

#include "SimpleIni.h"
#include "../widgets/richtextdelegate.h"

namespace twtgui {

twtgui::ViewFeed::ViewFeed(QWidget *parent, std::string configFile)
    : QWidget(parent), configFile(configFile)
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
    tweetsView->setMinimumHeight(512);
    tweetsView->setMinimumWidth(512);
    tweetsView->setItemDelegate(new RichTextDelegate(this));

    mainLayout->addWidget(tweetsView);
    mainLayout->addWidget(refreshButton);
    setLayout(mainLayout);

    refreshTimeline(); // initial load
}

void twtgui::ViewFeed::refreshTimeline(std::string username, std::string twtxtFeedString)
{
    tweetsModel->clear();

    std::stringstream text(twtxtFeedString);

    std::string line;
    std::vector<std::pair<QDateTime, std::string>> tweets;

    while (std::getline(text, line))
    {
        size_t tab = line.find('\t');
        if (tab == std::string::npos)
            continue;

        std::string timestamp = line.substr(0, tab);
        std::string value = line.substr(tab + 1);

        QDateTime dt = QDateTime::fromString(QString::fromStdString(timestamp), Qt::ISODate);
        tweets.emplace_back(dt, value);
    }

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

void twtgui::ViewFeed::handleButtonClick()
{
    qDebug() << "Refresh button clicked!";
    refreshTimeline();
}

twtgui::ViewFeed::~ViewFeed() {}

} // namespace twtgui