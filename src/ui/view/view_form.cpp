#include "view_form.h"
#include "download.h"

#include <fstream>
#include <sstream>
#include <iostream>

#include <QDateTime>
#include <QTimeZone>
#include <QDebug>

namespace twtgui {

twtgui::ViewForm::ViewForm(QWidget *parent, ViewFeed* viewFeed)
    : QWidget(parent)
{
    this->viewFeed = viewFeed;
    QHBoxLayout* containerLayout = new QHBoxLayout(this);
    field = new QLineEdit(this);
    viewButton = new QPushButton("View", this);

    field->setPlaceholderText("https://example.com/twtxt.txt");

    containerLayout->addWidget(field);
    containerLayout->addWidget(viewButton);

    setLayout(containerLayout);

    

    connect(field, &QLineEdit::returnPressed, this, &ViewForm::handleButtonClick);
    connect(viewButton, &QPushButton::clicked, this, &ViewForm::handleButtonClick);
}

void twtgui::ViewForm::handleButtonClick()
{
    if (field->text().isEmpty()) {
        return;
    }

    // get username for view feed

    std::vector<std::string> strings;
    std::istringstream ss(field->text().toStdString());
    std::string string;

    while (std::getline(ss, string, '/')) {
        strings.push_back(string);
    }

    TwtDownloader downloader;
    std::string outString = "";
    TwtDownloader::Result result = downloader.downloadToString(field->text().toStdString(), outString, 30, true);
    if (!result.success) {
        qDebug() << "Download failed:" << QString::fromStdString(result.error);
        field->setText("Download failed: " + QString::fromStdString(result.error));
        return;
    }

    viewFeed->refreshTimeline(strings[2], outString);
    
    return;
}


twtgui::ViewForm::~ViewForm() {}

} // namespace twtgui