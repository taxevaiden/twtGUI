#ifndef TWEETFORM_H
#define TWEETFORM_H

#include "timeline.h"
#include <string>
#include <QHBoxLayout>
#include <QLineEdit>
#include <QPushButton>

#include "SimpleIni.h"

namespace twtgui {

class TweetForm : public QWidget
{
    Q_OBJECT

    public:
        TweetForm(QWidget *parent = nullptr, Timeline* timeline = nullptr);
        ~TweetForm();
    private slots:
        void handleButtonClick();
    private:
        std::string twtxtFile;
        CSimpleIniA config;
        QLineEdit* field;
        QPushButton* postButton;
        Timeline* timeline;
};

} // namespace twtgui

using TweetForm = twtgui::TweetForm;

#endif // TWEETFORM_H