#include "ui/window.h"
#include "ui/tweet_form.h"
#include "ui/timeline.h"

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

#include "SimpleIni.h"

int main(int argc, char *argv[])
{
    QApplication a(argc, argv);

    CSimpleIniA config;
    SI_Error rc = config.LoadFile("C:\\Users\\aiden\\AppData\\Roaming\\twtxt\\config");

    if (rc < 0)
    {
        qDebug() << "Error: Could not open config.ini file";
        return 1;
    }

    MainWindow window;
    QString username = config.GetValue("twtxt", "nick", "unknown");
    qDebug() << "Current username:" << username;

    window.setWindowTitle("twtGUI - " + username);

    QTabWidget centralWidget (&window);


    // TIMELINE TAB

    QWidget timelineWidget;
    QVBoxLayout timelineLayout;

    Timeline timeline (&timelineWidget, "C:\\Users\\aiden\\AppData\\Roaming\\twtxt\\config");
    TweetForm tweetForm(&timelineWidget, &timeline, config.GetValue("twtxt", "twtfile", "unknown"));

    timelineLayout.addWidget(&tweetForm);
    timelineLayout.addWidget(&timeline);

    timelineWidget.setLayout(&timelineLayout);

    centralWidget.addTab(&timelineWidget, "Timeline");

    window.setCentralWidget(&centralWidget);
    window.show();

    return a.exec();
}
