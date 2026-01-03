#ifndef VIEWFEED_H
#define VIEWFEED_H

#include <string>
#include <QPushButton>
#include <QVBoxLayout>

namespace twtgui {

class ViewFeed : public QWidget
{
    Q_OBJECT
    public:
        ViewFeed(QWidget *parent = nullptr, std::string configFile = "");
        ~ViewFeed();
        void refreshTimeline(std::string username = "", std::string twtxtFeedString = "");
    private slots:
        void handleButtonClick();
    private:
        QPushButton* refreshButton;
        std::string configFile;
        QVBoxLayout* mainLayout;
        QVBoxLayout* tweetsLayout;
};

} // namespace twtgui

using ViewFeed = twtgui::ViewFeed;

#endif // VIEWFEED_H