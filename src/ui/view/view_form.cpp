#include "view_form.h"

#include <fstream>
#include <sstream>
#include <iostream>

#include <QDateTime>
#include <QTimeZone>
#include <QDebug>

#include "SimpleIni.h"

namespace twtgui {

twtgui::ViewForm::ViewForm(QWidget *parent, std::string configFile, ViewFeed* viewFeed)
    : QWidget(parent)
{
    this->configFile = configFile;
    this->viewFeed = viewFeed;
    QHBoxLayout* containerLayout = new QHBoxLayout(this);
    field = new QLineEdit(this);
    followButton = new QPushButton("You!", this);
    followButton->setEnabled(false); // views your own feed by default, so we disable
    viewButton = new QPushButton("View", this);

    field->setPlaceholderText("https://example.com/twtxt.txt");

    containerLayout->addWidget(field);
    containerLayout->addWidget(followButton);
    containerLayout->addWidget(viewButton);

    setLayout(containerLayout);

    

    connect(field, &QLineEdit::returnPressed, this, &ViewForm::handleViewButtonClick);
    connect(viewButton, &QPushButton::clicked, this, &ViewForm::handleViewButtonClick);
}

void twtgui::ViewForm::handleViewButtonClick()
{
    if (field->text().isEmpty()) {
        return;
    }

    CSimpleIniA config;
    SI_Error rc = config.LoadFile(configFile.c_str());
    
    CSimpleIniA::TNamesDepend keys;

    config.GetAllKeys("following", keys);
    CSimpleIniA::TNamesDepend::const_iterator it;
    for (it = keys.begin(); it != keys.end(); ++it) {
        const char* key = it->pItem;
        const char* value = config.GetValue("following", key, nullptr);
        if (value != nullptr) {
            QString urlStr = QString::fromStdString(field->text().toStdString());
            QString valueStr = QString::fromStdString(value);
            if (urlStr == valueStr) {
                followButton->setText("Unfollow");
                followButton->setEnabled(true); // views a feed you follow, so we enable
                break;
            } else if (urlStr == config.GetValue("twtxt", "twturl", nullptr)) {
                followButton->setText("You!");
                followButton->setEnabled(false); // views your own feed, so we disable
                break;
            } else if (urlStr != valueStr) {
                followButton->setText("Follow");
                followButton->setEnabled(true); // views a feed you don't follow, so we enable
            }
        }
    }

    // pass the URL to the feed
    viewFeed->refreshTimeline(field->text().toStdString());
    return;
}

void twtgui::ViewForm::handleFollowButtonClick()
{
    if (field->text().isEmpty()) {
        return;
    }
}


twtgui::ViewForm::~ViewForm() {}

} // namespace twtgui