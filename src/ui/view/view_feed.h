#ifndef VIEWFEED_H
#define VIEWFEED_H

#include <string>
#include <QPushButton>
#include <QVBoxLayout>
#include <QListView>
#include <QStandardItemModel>

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
        QListView* tweetsView;
        QStandardItemModel* tweetsModel;
};

} // namespace twtgui

using ViewFeed = twtgui::ViewFeed;

#endif // VIEWFEED_H