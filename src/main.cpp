#include "ui/window.h"
#include "ui/timeline/tweet_form.h"
#include "ui/timeline/timeline.h"

#include "ui/view/view_form.h"
#include "ui/view/view_feed.h"

#include <iostream>
#include <fstream>

#include <QDir>
#include <QString>
#include <QDebug>

#include <QApplication>
#include <QHBoxLayout>
#include <QVBoxLayout>
#include <QLabel>
#include <QLineEdit>
#include <QPushButton>
#include <QTabWidget>
#include <QMessageBox>
#include <QCoreApplication>
#include <QLibraryInfo>
#include <QFile>

#include "SimpleIni.h"

int main(int argc, char *argv[])
{

    QApplication a(argc, argv);

    CSimpleIniA config;
    SI_Error rc = config.LoadFile((QDir::homePath().toStdString() + "\\AppData\\Roaming\\twtxt\\config").c_str());

    if (rc < 0)
    {
        qDebug() << "Warning: Could not open config file; proceeding with defaults.";
        // Continue with defaults; CSimpleIni will return default values where used.
    }

    twtgui::MainWindow window;
    QString username = config.GetValue("twtxt", "nick", "unknown");
    qDebug() << "Current username:" << username;

    window.setWindowTitle("twtGUI - " + username);

    // Create central widget and child widgets on the heap so Qt manages their lifetime
    QTabWidget *centralWidget = new QTabWidget(&window);

    // TIMELINE TAB
    QWidget *timelineWidget = new QWidget();
    QVBoxLayout *timelineLayout = new QVBoxLayout(timelineWidget);

    twtgui::Timeline *timeline = new twtgui::Timeline(timelineWidget, (QDir::homePath().toStdString() + "\\AppData\\Roaming\\twtxt\\config").c_str());
    twtgui::TweetForm *tweetForm = new twtgui::TweetForm(timelineWidget, timeline, config.GetValue("twtxt", "twtfile", "unknown"));

    timelineLayout->addWidget(tweetForm);
    timelineLayout->addWidget(timeline);

    timelineWidget->setLayout(timelineLayout);
    centralWidget->addTab(timelineWidget, "Timeline");

    // VIEW TAB
    QWidget *viewWidget = new QWidget();
    QVBoxLayout *viewLayout = new QVBoxLayout(viewWidget);

    twtgui::ViewFeed *viewFeed = new twtgui::ViewFeed(viewWidget, (QDir::homePath().toStdString() + "\\AppData\\Roaming\\twtxt\\config").c_str());
    twtgui::ViewForm *viewForm = new twtgui::ViewForm(viewWidget, (QDir::homePath().toStdString() + "\\AppData\\Roaming\\twtxt\\config").c_str(), viewFeed);

    viewLayout->addWidget(viewForm);
    viewLayout->addWidget(viewFeed);

    viewWidget->setLayout(viewLayout);

    centralWidget->addTab(viewWidget, "View");

    //FOLLWING TAB
    QWidget *followingWidget = new QWidget();
    QVBoxLayout *followingLayout = new QVBoxLayout(followingWidget);

    CSimpleIniA::TNamesDepend keys;

    config.GetAllKeys("following", keys);
    CSimpleIniA::TNamesDepend::const_iterator it;
    for (it = keys.begin(); it != keys.end(); ++it) {
        QWidget *entryContainer = new QWidget();
        std::string followName = it->pItem;
        std::string followUrl = config.GetValue("following", followName.c_str(), "");

        QLabel *followLabel = new QLabel(QString::fromStdString("<b>" + followName + "</b> @ " + followUrl), entryContainer);
        QPushButton *unfollowButton = new QPushButton("Unfollow", entryContainer);
        QPushButton *viewButton = new QPushButton("View", entryContainer);
        QHBoxLayout *entryLayout = new QHBoxLayout(entryContainer);
        entryLayout->addWidget(followLabel);
        entryLayout->addStretch();
        entryLayout->addWidget(unfollowButton);
        entryLayout->addWidget(viewButton);
        entryContainer->setLayout(entryLayout);
        followingLayout->addWidget(entryContainer);
    }

    followingLayout->addStretch();

    followingWidget->setLayout(followingLayout);
    centralWidget->addTab(followingWidget, "Following");

    window.setCentralWidget(centralWidget);

    window.show();

    return a.exec();
}
