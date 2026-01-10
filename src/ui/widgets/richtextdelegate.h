#ifndef RICHTEXTDELEGATE_H
#define RICHTEXTDELEGATE_H

#include <QStyledItemDelegate>
#include <QTextDocument>

class RichTextDelegate : public QStyledItemDelegate
{
    public:
        explicit RichTextDelegate(QObject *parent = nullptr) : QStyledItemDelegate(parent) {}
        void paint(QPainter *painter, const QStyleOptionViewItem &option, const QModelIndex &index) const override;
        QSize sizeHint(const QStyleOptionViewItem &option, const QModelIndex &index) const override;
    private: 
        mutable QTextDocument doc;
};
    
#endif // RICHTEXTDELEGATE_H
