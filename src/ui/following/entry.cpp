#include "entry.h"

#include "SimpleIni.h"
#include <QTabWidget>
#include <QDebug>
#include <QDir>
#include <QDialog>
#include <QDialogButtonBox>
#include <QFormLayout>
#include <QLineEdit>
#include <QLabel>
#include "../../config.h"

namespace twtgui
{
    twtgui::FollowingEntry::FollowingEntry(QWidget *parent, const std::string &name, const std::string &url, ViewFeed *viewFeed, QTabWidget *parentTabWidget)
        : QWidget(parent), name(name), url(url), viewFeed(viewFeed), parentTabWidget(parentTabWidget)
    {
        this->config.LoadFile("config.ini");

        entryLayout = new QHBoxLayout(this);
        followLabel = new QLabel(QString::fromStdString("<b>" + name + "</b> @ " + url), this);
        // unfollowButton = new QPushButton("Unfollow", this);
        // editButton = new QPushButton("Edit", this);
        // viewButton = new QPushButton("View", this);
        menuButton = new QPushButton("...", this);

        menu = new QMenu(this);
        viewAction = menu->addAction("View");
        editAction = menu->addAction("Edit");
        unfollowAction = menu->addAction("Unfollow");

        menuButton->setMenu(menu);

        entryLayout->addWidget(followLabel);
        entryLayout->addStretch();
        // entryLayout->addWidget(unfollowButton);
        // entryLayout->addWidget(editButton);
        // entryLayout->addWidget(viewButton);
        entryLayout->addWidget(menuButton);

        setLayout(entryLayout);

        connect(unfollowAction, &QAction::triggered, this, &FollowingEntry::handleUnfollowButtonClick);
        connect(editAction, &QAction::triggered, this, &FollowingEntry::handleEditButtonClick);
        connect(viewAction, &QAction::triggered, this, &FollowingEntry::handleViewButtonClick);
    }

    twtgui::FollowingEntry::~FollowingEntry()
    {
    }

    void twtgui::FollowingEntry::handleUnfollowButtonClick()
    {

        twtgui::GlobalConfig::config.Delete("following", name.c_str());
        SI_Error rc = twtgui::GlobalConfig::config.SaveFile("config.ini");
        if (rc < 0)
        {
            qDebug() << "Error saving config file:" << rc;
            return;
        }

        // remove this entry from the UI
        this->deleteLater();
    }

    void twtgui::FollowingEntry::handleEditButtonClick()
    {
        QDialog *dlg = new QDialog(this);
        dlg->setWindowTitle("Editing " + QString::fromStdString(name));

        // form layout
        QFormLayout *formLayout = new QFormLayout(dlg);

        QLineEdit *nickField = new QLineEdit(QString::fromStdString(name), dlg);
        QLineEdit *urlField = new QLineEdit(QString::fromStdString(url), dlg);

        // buttons
        QDialogButtonBox *buttons =
            new QDialogButtonBox(QDialogButtonBox::Ok | QDialogButtonBox::Cancel, dlg);

        formLayout->addRow("Nickname", nickField);
        formLayout->addRow("URL", urlField);
        formLayout->addRow(buttons);

        // outer layout to place form + buttons
        dlg->setLayout(formLayout);

        connect(buttons, &QDialogButtonBox::accepted, dlg, &QDialog::accept);
        connect(buttons, &QDialogButtonBox::rejected, dlg, &QDialog::reject);

        dlg->exec();

        if (dlg->result() == QDialog::Accepted)
        {
            std::string newNick = nickField->text().toStdString();
            std::string newUrl = urlField->text().toStdString();

            // remove old entry
            twtgui::GlobalConfig::config.Delete("following", name.c_str());

            // add new entry
            twtgui::GlobalConfig::config.SetValue(
                "following",
                newNick.c_str(),
                newUrl.c_str());

            SI_Error rc = twtgui::GlobalConfig::config.SaveFile("config.ini");
            if (rc < 0)
            {
                qDebug() << "Error saving config file:" << rc;
                return;
            }

            // update UI
            name = newNick;
            url = newUrl;
            followLabel->setText(
                QString::fromStdString("<b>" + name + "</b> @ " + url));
        }
    }

    void twtgui::FollowingEntry::handleViewButtonClick()
    {
        if (viewFeed != nullptr)
        {
            viewFeed->refreshTimeline(url);
            parentTabWidget->setCurrentIndex(1);
        }
    }
}