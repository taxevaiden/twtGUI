#include "timeline.h"

#include <iostream>
#include <fstream>

#include <QScrollArea>
#include <QDateTime>
#include <QTimeZone>
#include <QVBoxLayout>
#include <QLabel>
#include <QDebug>

#include "SimpleIni.h"

namespace twtgui {

twtgui::Timeline::Timeline(QWidget *parent, std::string configFile)
    : QWidget(parent), configFile(configFile)
{
    mainLayout = new QVBoxLayout(this);

    // refresh button
    refreshButton = new QPushButton("Refresh", this);
    connect(refreshButton, &QPushButton::clicked, this, &Timeline::handleButtonClick);
   
    // container layout for tweets
    tweetsLayout = new QVBoxLayout();
    QWidget *tweetsContainer = new QWidget();
    tweetsContainer->setLayout(tweetsLayout);

    QScrollArea *scrollArea = new QScrollArea(this);
    scrollArea->setWidget(tweetsContainer);
    scrollArea->setWidgetResizable(true);
    scrollArea->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
    scrollArea->setMinimumHeight(512);
    scrollArea->setMinimumWidth(512);

    mainLayout->addWidget(scrollArea);
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

    QLabel *tweetLabel = new QLabel(
        dt.toString("MM-dd-yyyy hh:mm AP") + " <b>" + QString::fromStdString(username) + "</b>: " + QString::fromStdString(content),
        this);
    tweetLabel->setWordWrap(true);
    tweetsLayout->insertWidget(0, tweetLabel);
}

void twtgui::Timeline::refreshTimeline()
{
    QLayoutItem *item;
    while ((item = tweetsLayout->takeAt(0)) != nullptr)
    {
        delete item->widget();
        delete item;
    }

    CSimpleIniA config;
    config.LoadFile(configFile.c_str());
    std::string username = config.GetValue("twtxt", "nick", "unknown");

    std::ifstream file(config.GetValue("twtxt", "twtfile", ""));
    if (!file.is_open())
    {
        QLabel *errorLabel = new QLabel("Could not open twtxt file.", this);
        tweetsLayout->addWidget(errorLabel);
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
        QLabel *tweetLabel = new QLabel(
            tweet.first.toString("MM-dd-yyyy hh:mm AP") + " <b>" + QString::fromStdString(username) + "</b>: " + QString::fromStdString(tweet.second),
            this);
        tweetLabel->setWordWrap(true);
        tweetsLayout->insertWidget(0, tweetLabel);
    }

    tweetsLayout->addStretch();
}

void twtgui::Timeline::handleButtonClick()
{
    qDebug() << "Refresh button clicked!";
    refreshTimeline();
}

twtgui::Timeline::~Timeline() {}

} // namespace twtgui