#include "richtextdelegate.h"

#include <QTextDocument>
#include <QPainter>
#include <QApplication>
#include <QStyle>
#include <QListView>
#include <algorithm>


void RichTextDelegate::paint(QPainter *painter, const QStyleOptionViewItem &option, const QModelIndex &index) const
{
    QStyleOptionViewItem opt(option);
    initStyleOption(&opt, index);

    const QWidget *cwidget = opt.widget;
    QWidget *widget = const_cast<QWidget*>(cwidget);
    QStyle *style = widget ? widget->style() : QApplication::style();

    // Draw the standard item background/selection
    style->drawPrimitive(QStyle::PE_PanelItemViewItem, &opt, painter, widget);

    QTextDocument doc;
    doc.setHtml(index.data(Qt::DisplayRole).toString());

    int textWidth = opt.rect.width();
    const QListView *view = qobject_cast<const QListView*>(opt.widget);
    if (textWidth <= 0) {
        if (view)
            textWidth = view->viewport()->width();
        else if (widget)
            textWidth = widget->width();
    }

    const int padding = 8;
    textWidth = std::max(1, textWidth - padding);
    doc.setTextWidth(textWidth);

    // draw text within the rect minus padding
    QRect textRect = opt.rect.adjusted(4, 4, -4, -4);
    painter->save();
    painter->translate(textRect.topLeft());
    QRect clip(0, 0, textWidth, std::max(1, (int)std::ceil(doc.size().height())));
    doc.drawContents(painter, clip);
    painter->restore();
} 

QSize RichTextDelegate::sizeHint(const QStyleOptionViewItem &option, const QModelIndex &index) const
{
    QStyleOptionViewItem opt(option);
    initStyleOption(&opt, index);

    QTextDocument doc;
    doc.setHtml(index.data(Qt::DisplayRole).toString());

    int textWidth = opt.rect.width();
    const QListView *view = qobject_cast<const QListView*>(opt.widget);
    if (textWidth <= 0) {
        if (view)
            textWidth = view->viewport()->width();
        else if (opt.widget)
            textWidth = opt.widget->width();
    }

    const int padding = 8;
    textWidth = std::max(1, textWidth - padding);

    doc.setTextWidth(textWidth);
    QSizeF s = doc.size();

    // Add vertical padding
    return QSize(textWidth, (int)std::ceil(s.height()) + 8);
}
