#include "ui/window.h"
#include "ui/timeline/tweet_form.h"
#include "ui/timeline/timeline.h"

#include "ui/view/view_form.h"
#include "ui/view/view_feed.h"

#include "ui/following/entry.h"

#include "ui/settings/panel.h"

#include "config.h"

#include <iostream>
#include <fstream>

#include <QDir>
#include <QString>
#include <QDebug>

#include <QApplication>
#include <QHBoxLayout>
#include <QVBoxLayout>
#include <QFormLayout>
#include <QLabel>
#include <QLineEdit>
#include <QPushButton>
#include <QTabWidget>
#include <QMessageBox>
#include <QCoreApplication>
#include <QLibraryInfo>
#include <QFile>
#include <QFileDialog>
#include <QScrollArea>

#include <QDialog>
#include <QDialogButtonBox>

#include "SimpleIni.h"

// TODO: cleanup entire project

int main(int argc, char *argv[])
{
    QApplication a(argc, argv);

    if (std::filesystem::exists("config.ini"))
    {
        twtgui::GlobalConfig::loadConfig("config.ini");
    }
    else
    {
        QDialog *dlg = new QDialog();

        dlg->setWindowTitle("Initial Setup");

        QFormLayout *layout = new QFormLayout(dlg);
        QLabel *nickLabel = new QLabel("Nickname", dlg);
        QLabel *twtxtLabel = new QLabel("twtxt", dlg);
        QLabel *urlLabel = new QLabel("URL", dlg);

        QLineEdit *nickField = new QLineEdit("", dlg);
        QLineEdit *twtxtField = new QLineEdit("", dlg);
        QLineEdit *urlField = new QLineEdit("", dlg);

        QPushButton *browseButton = new QPushButton("Browse", dlg);

        dlg->connect(browseButton, &QPushButton::clicked, [dlg, twtxtField]()
                     {
            QString filePath = QFileDialog::getOpenFileName(dlg, "Select File", "", "All Files (*)");
                if (!filePath.isEmpty()) {
                    twtxtField->setText(filePath);
                } });

        QHBoxLayout *twtxtLayout = new QHBoxLayout();
        twtxtLayout->addWidget(twtxtField);
        twtxtLayout->addWidget(browseButton);

        QDialogButtonBox *buttons =
            new QDialogButtonBox(QDialogButtonBox::Ok, dlg);

        layout->addRow(nickLabel, nickField);
        layout->addRow(twtxtLabel, twtxtLayout);
        layout->addRow(urlLabel, urlField);
        layout->addRow(buttons);

        dlg->setLayout(layout);

        dlg->connect(buttons, &QDialogButtonBox::accepted, dlg, &QDialog::accept);

        // connect(buttons, &QDialogButtonBox::accepted, dlg, &QDialog::accept);

        dlg->exec();

        if (dlg->result() == QDialog::Accepted)
        {
            std::string newNick = nickField->text().toStdString();
            std::string newTwtxt = twtxtField->text().toStdString();
            std::string newUrl = urlField->text().toStdString();

            twtgui::GlobalConfig::config.SetValue("settings", "nick", newNick.c_str());
            twtgui::GlobalConfig::config.SetValue("settings", "twtxt", newTwtxt.c_str());
            twtgui::GlobalConfig::config.SetValue("settings", "twturl", newUrl.c_str());
            twtgui::GlobalConfig::config.SaveFile("config.ini");

            SI_Error rc = twtgui::GlobalConfig::config.SaveFile("config.ini");
            if (rc < 0)
            {
                qDebug() << "Error saving config file:" << rc;
                return rc;
            }
        }
    }

    twtgui::MainWindow window;
    QString username = twtgui::GlobalConfig::config.GetValue("settings", "nick", "unknown");
    qDebug() << "Current username:" << username;

    window.setWindowTitle("twtGUI - " + username);

    // create central widget and child widgets on the heap so Qt manages their lifetime
    QTabWidget *centralWidget = new QTabWidget(&window);

    // TIMELINE TAB
    QWidget *timelineWidget = new QWidget();
    QVBoxLayout *timelineLayout = new QVBoxLayout(timelineWidget);

    twtgui::Timeline *timeline = new twtgui::Timeline(timelineWidget);
    twtgui::TweetForm *tweetForm = new twtgui::TweetForm(timelineWidget, timeline);

    timelineLayout->addWidget(tweetForm);
    timelineLayout->addWidget(timeline);

    timelineWidget->setLayout(timelineLayout);
    centralWidget->addTab(timelineWidget, "Timeline");

    // VIEW TAB
    QWidget *viewWidget = new QWidget();
    QVBoxLayout *viewLayout = new QVBoxLayout(viewWidget);
    twtgui::ViewFeed *viewFeed = new twtgui::ViewFeed(viewWidget);
    twtgui::ViewForm *viewForm = new twtgui::ViewForm(viewWidget, viewFeed);

    viewLayout->addWidget(viewForm);
    viewLayout->addWidget(viewFeed);

    viewWidget->setLayout(viewLayout);

    centralWidget->addTab(viewWidget, "View");

    // FOLLOWING TAB
    QWidget *followingWidget = new QWidget();
    QVBoxLayout *followingLayout = new QVBoxLayout(followingWidget);

    QWidget *followingContainer = new QWidget();
    QVBoxLayout *followingContainerLayout = new QVBoxLayout(followingContainer);

    QScrollArea *scrollArea = new QScrollArea(followingWidget);
    scrollArea->setWidget(followingContainer);
    scrollArea->setWidgetResizable(true);
    scrollArea->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
    scrollArea->setMinimumWidth(512);
    scrollArea->setMinimumHeight(512);

    CSimpleIniA::TNamesDepend keys;

    twtgui::GlobalConfig::config.GetAllKeys("following", keys);
    CSimpleIniA::TNamesDepend::const_iterator it;
    for (it = keys.begin(); it != keys.end(); ++it)
    {
        std::string followName = it->pItem;
        std::string followUrl = twtgui::GlobalConfig::config.GetValue("following", followName.c_str(), "");

        twtgui::FollowingEntry *entry = new twtgui::FollowingEntry(followingContainer, followName, followUrl, viewFeed, centralWidget);

        followingContainerLayout->addWidget(entry);
    }

    followingContainerLayout->addStretch();

    QPushButton *refreshButton = new QPushButton("Refresh", followingWidget);

    followingContainer->connect(refreshButton, &QPushButton::clicked, followingContainer, [followingContainerLayout, followingContainer, viewFeed, centralWidget]()
    {
        QLayoutItem *child;
        while ((child = followingContainerLayout->takeAt(0)) != 0) {
            if (child->widget()) {
                delete child->widget(); // Deletes the widget instance
            }
        delete child; // Deletes the layout item
        }

        CSimpleIniA::TNamesDepend keys;

        twtgui::GlobalConfig::config.GetAllKeys("following", keys);
        CSimpleIniA::TNamesDepend::const_iterator it;
        for (it = keys.begin(); it != keys.end(); ++it)
        {
            std::string followName = it->pItem;
            std::string followUrl = twtgui::GlobalConfig::config.GetValue("following", followName.c_str(), "");

            twtgui::FollowingEntry *entry = new twtgui::FollowingEntry(followingContainer, followName, followUrl, viewFeed, centralWidget);

            followingContainerLayout->addWidget(entry);
        }

        followingContainerLayout->addStretch();
    });


    followingLayout->addWidget(scrollArea);
    followingLayout->addWidget(refreshButton);

    followingContainer->setLayout(followingContainerLayout);
    followingWidget->setLayout(followingLayout);
    centralWidget->addTab(followingWidget, "Following");

    // SETTINGS TAB

    twtgui::SettingsPanel *settingsPanel = new twtgui::SettingsPanel(centralWidget);
    settingsPanel->addSetting("Nickname", "Your nickname that will be displayed.", "nick", twtgui::SettingType::SettingType_Text);
    settingsPanel->addSetting("twtxt.txt", "Path to the twtxt.txt file.", "twtxt", twtgui::SettingType::SettingType_FilePath);
    settingsPanel->addSetting("twtxt.txt URL", "URL to your public twtxt.txt.", "twturl", twtgui::SettingType::SettingType_Text);

    // pre/post-script will be implemented soon but i cannot be bothered to do it now i'm so tired
    // settingsPanel->addSetting("Pre-script", "Path to a script that executes BEFORE tweeting.", "pre-script", twtgui::SettingType::SettingType_FilePath);
    // settingsPanel->addSetting("Post-script", "Path to a script that executes AFTER tweeting.", "post-script", twtgui::SettingType::SettingType_FilePath);

    settingsPanel->addSetting("Colored names", "Whether the names on the timeline will be colored.", "colored_names", twtgui::SettingType::SettingType_Check);

    centralWidget->addTab(settingsPanel, "Settings");

    window.setCentralWidget(centralWidget);

    window.show();

    return a.exec();
}
