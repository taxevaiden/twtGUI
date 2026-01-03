#ifndef TWEETFORM_H
#define TWEETFORM_H

#include "timeline.h"
#include <string>
#include <QHBoxLayout>
#include <QLineEdit>
#include <QPushButton>

namespace twtgui {

class TweetForm : public QWidget
{
    Q_OBJECT

    public:
        TweetForm(QWidget *parent = nullptr, Timeline* timeline = nullptr, std::string twtxtFile = "");
        ~TweetForm();
    private slots:
        void handleButtonClick();
    private:
        std::string twtxtFile;
        QLineEdit* field;
        QPushButton* postButton;
        Timeline* timeline;
};

} // namespace twtgui

using TweetForm = twtgui::TweetForm;

#endif // TWEETFORM_H