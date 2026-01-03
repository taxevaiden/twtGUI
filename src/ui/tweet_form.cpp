#include "tweet_form.h"
#include "timeline.h"

#include <fstream>
#include <iostream>

#include <QDateTime>
#include <QTimeZone>
#include <QDebug>

TweetForm::TweetForm(QWidget *parent, Timeline* timeline, std::string twtxtFile)
    : QWidget(parent)
{
    this->twtxtFile = twtxtFile;
    this->timeline = timeline;
    
    QHBoxLayout* containerLayout = new QHBoxLayout(this);
    field = new QLineEdit(this);
    postButton = new QPushButton("Post", this);

    field->setPlaceholderText("What's on your mind?");

    containerLayout->addWidget(field);
    containerLayout->addWidget(postButton);

    setLayout(containerLayout);

    connect(field, &QLineEdit::returnPressed, this, &TweetForm::handleButtonClick);
    connect(postButton, &QPushButton::clicked, this, &TweetForm::handleButtonClick);
}

void TweetForm::handleButtonClick()
{
    if (field->text().isEmpty()) {
        return;
    }

    std::ofstream outFile(twtxtFile, std::ios::app);

    if (!outFile.is_open()) {
        qDebug() << "Error opening file!";
        return;
    }

    outFile 
        << QDateTime::currentDateTime()
            .toTimeZone(QTimeZone::systemTimeZone())
            .toString(Qt::ISODate).toStdString()
        << "\t" 
        << field->text()
            .toStdString() 
        << std::endl; 

    field->clear();

    outFile.close();

    timeline->refreshTimeline();
    return;
}

TweetForm::~TweetForm() {}