#ifndef VIEWFORM_H
#define VIEWFORM_H

#include "view_feed.h"
#include <string>
#include <QHBoxLayout>
#include <QLineEdit>
#include <QPushButton>

#include "SimpleIni.h"

namespace twtgui {

class ViewForm : public QWidget
{
    Q_OBJECT

    public:
        ViewForm(QWidget *parent = nullptr, ViewFeed* viewFeed = nullptr);
        ~ViewForm();
    private slots:
        void handleFollowButtonClick();
        void handleViewButtonClick();
    private:
        std::string configFile;
        CSimpleIniA config;
        ViewFeed* viewFeed;
        QLineEdit* field;
        QPushButton* followButton;
        QPushButton* viewButton;
};

} // namespace twtgui

using ViewForm = twtgui::ViewForm;

#endif // VIEWFORM_H