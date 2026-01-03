#ifndef RICHTEXTDELEGATE_H
#define RICHTEXTDELEGATE_H

#include <QStyledItemDelegate>

class RichTextDelegate : public QStyledItemDelegate
{
public:
    explicit RichTextDelegate(QObject *parent = nullptr) : QStyledItemDelegate(parent) {}
    void paint(QPainter *painter, const QStyleOptionViewItem &option, const QModelIndex &index) const override;
    QSize sizeHint(const QStyleOptionViewItem &option, const QModelIndex &index) const override;
};

#endif // RICHTEXTDELEGATE_H
