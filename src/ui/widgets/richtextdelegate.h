#ifndef RICHTEXTDELEGATE_H
#define RICHTEXTDELEGATE_H

#include <QStyledItemDelegate>
#include <QTextDocument>
#include <QSharedPointer>
#include <QPair>

class RichTextDelegate : public QStyledItemDelegate
{
    public:
        explicit RichTextDelegate(QObject *parent = nullptr);
        void paint(QPainter *painter, const QStyleOptionViewItem &option, const QModelIndex &index) const override;
        QSize sizeHint(const QStyleOptionViewItem &option, const QModelIndex &index) const override;
        bool eventFilter(QObject *obj, QEvent *event);
    
    private:
        struct CachedDoc
        {
            QSharedPointer<QTextDocument> doc;
            int width = -1;
        };
    
        mutable QHash<QPair<QPersistentModelIndex, int>, int> heightCache;
        mutable QHash<QPersistentModelIndex, CachedDoc> docCache;
    
        QTextDocument *documentFor(const QModelIndex &index,
                                   int width) const;
};

#endif // RICHTEXTDELEGATE_H
