#ifndef VIEWFORM_H
#define VIEWFORM_H

#include "view_feed.h"
#include <string>
#include <QHBoxLayout>
#include <QLineEdit>
#include <QPushButton>

namespace twtgui {

class ViewForm : public QWidget
{
    Q_OBJECT

    public:
        ViewForm(QWidget *parent = nullptr, std::string configFile = "", ViewFeed* viewFeed = nullptr);
        ~ViewForm();
    private slots:
        void handleFollowButtonClick();
        void handleViewButtonClick();
    private:
        std::string configFile;
        ViewFeed* viewFeed;
        QLineEdit* field;
        QPushButton* followButton;
        QPushButton* viewButton;
};

} // namespace twtgui

using ViewForm = twtgui::ViewForm;

#endif // VIEWFORM_H