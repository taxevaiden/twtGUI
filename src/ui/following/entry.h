#ifndef ENTRY_H
#define ENTRY_H

#include <string>

#include "../view/view_feed.h"

#include <QTabWidget>
#include <QWidget>
#include <QLabel>
#include <QAction>
#include <QPushButton>
#include <QComboBox>
#include <QMenu>
#include <QHBoxLayout>

#include "SimpleIni.h"

namespace twtgui
{
    class FollowingEntry : public QWidget
    {
        Q_OBJECT

        public:
            FollowingEntry(QWidget *parent = nullptr, const std::string &name = "", const std::string &url = "", ViewFeed *viewFeed = nullptr, QTabWidget *parentTabWidget = nullptr);
            ~FollowingEntry();
        private slots:
            void handleUnfollowButtonClick();
            void handleEditButtonClick();
            void handleViewButtonClick();
        private:
            std::string name;
            std::string url;

            CSimpleIniA config;

            ViewFeed *viewFeed;
            QTabWidget *parentTabWidget;
            QLabel *followLabel;
            QMenu *menu;
            QPushButton *menuButton;
            QAction *unfollowAction;
            QAction *editAction;
            QAction *viewAction;
            QHBoxLayout *entryLayout;
        };
}

using FollowingEntry = twtgui::FollowingEntry;

#endif // ENTRY_H