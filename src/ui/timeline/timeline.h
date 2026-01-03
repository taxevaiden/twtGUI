#ifndef TIMELINE_H
#define TIMELINE_H

#include <string>
#include <QPushButton>
#include <QVBoxLayout>

namespace twtgui {

class Timeline : public QWidget
{
    Q_OBJECT
    public:
        Timeline(QWidget *parent = nullptr, std::string configFile = "");
        ~Timeline();
        void addTweet(std::string timestamp, std::string content);
        void refreshTimeline();
    private slots:
        void handleButtonClick();
    private:
        QPushButton* refreshButton;
        std::string configFile;
        QVBoxLayout* mainLayout;
        QVBoxLayout* tweetsLayout;
};

} // namespace twtgui

using Timeline = twtgui::Timeline;

#endif // TIMELINE_H