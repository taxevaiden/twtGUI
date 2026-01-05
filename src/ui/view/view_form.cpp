#include "view_form.h"

#include <fstream>
#include <sstream>
#include <iostream>

#include <QDateTime>
#include <QTimeZone>
#include <QDebug>

#include "SimpleIni.h"

#include "../../config.h"

namespace twtgui {

twtgui::ViewForm::ViewForm(QWidget *parent, ViewFeed* viewFeed)
    : QWidget(parent)
{
    this->config.LoadFile("config.ini");
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
    connect(followButton, &QPushButton::clicked, this, &ViewForm::handleFollowButtonClick);
    connect(viewButton, &QPushButton::clicked, this, &ViewForm::handleViewButtonClick);
}

void twtgui::ViewForm::handleViewButtonClick()
{
    if (field->text().isEmpty()) {
        return;
    }
    
    CSimpleIniA::TNamesDepend keys;

    twtgui::GlobalConfig::config.GetAllKeys("following", keys);
    CSimpleIniA::TNamesDepend::const_iterator it;
    for (it = keys.begin(); it != keys.end(); ++it) {
        const char* key = it->pItem;
        const char* value = twtgui::GlobalConfig::config.GetValue("following", key, nullptr);
        if (value != nullptr) {
            QString urlStr = field->text();
            QString valueStr = QString::fromStdString(value);
            if (urlStr == valueStr) {
                followButton->setText("Unfollow");
                followButton->setEnabled(true); // views a feed you follow, so we enable
                break;
            } else if (urlStr == twtgui::GlobalConfig::config.GetValue("settings", "twturl", nullptr)) {
                followButton->setText("You!");
                followButton->setEnabled(false); // views your own feed, so we disable
                break;
            } else if (urlStr != valueStr) {
                followButton->setText("Follow");
                followButton->setEnabled(true); // views a feed you don't follow, so we enable
            }
        }
    }

    if (keys.empty()) {
        if (field->text() == twtgui::GlobalConfig::config.GetValue("settings", "twturl", nullptr)) {
            followButton->setText("You!");
            followButton->setEnabled(false); // views your own feed, so we disable
        } else {
            followButton->setText("Follow");
            followButton->setEnabled(true); // views a feed you don't follow, so we enable
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

    // determine username: prefer the name (key) in the "following" section whose value equals
    // this URL; if not found, fall back to using the host part of the URL
    auto hostFromUrl = [](const std::string &u) -> std::string {
        if (u.empty()) return "";
        size_t pos = u.find("://");
        size_t start = (pos == std::string::npos) ? 0 : pos + 3;
        size_t end = u.find('/', start);
        return u.substr(start, end == std::string::npos ? std::string::npos : end - start);
    };

    std::string url = field->text().toStdString();

    std::string username = "";
    // search existing following keys for a matching value
    CSimpleIniA::TNamesDepend keys;
    twtgui::GlobalConfig::config.GetAllKeys("following", keys);
    for (CSimpleIniA::TNamesDepend::const_iterator it = keys.begin(); it != keys.end(); ++it) {
        const char* key = it->pItem;
        const char* value = twtgui::GlobalConfig::config.GetValue("following", key, nullptr);
        if (value != nullptr && url == value) {
            username = key;
            break;
        }
    }

    // fallback to host part if no key found
    if (username.empty()) {
        username = hostFromUrl(url);
    }
    SI_Error rc = 0;
    if (followButton->text() == "Follow") {
        // add to following
        twtgui::GlobalConfig::config.SetValue("following", username.c_str(), field->text().toStdString().c_str());
        rc = twtgui::GlobalConfig::config.SaveFile("config.ini");
        if (rc < 0) {
            qDebug() << "Error saving config file:" << rc;
            return;
        }
        followButton->setText("Unfollow");
    } else if (followButton->text() == "Unfollow") {
        // remove from following
        twtgui::GlobalConfig::config.Delete("following", username.c_str());
        rc = twtgui::GlobalConfig::config.SaveFile("config.ini");
        if (rc < 0) {
            qDebug() << "Error saving config file:" << rc;
            return;
        }
        followButton->setText("Follow");
    }
}


twtgui::ViewForm::~ViewForm() {}

} // namespace twtgui