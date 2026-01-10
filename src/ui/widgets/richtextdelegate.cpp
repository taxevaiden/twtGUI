#include "richtextdelegate.h"

#include <QTextDocument>
#include <QPainter>
#include <QApplication>
#include <QStyle>
#include <QListView>
#include <algorithm>

RichTextDelegate::RichTextDelegate(QObject *parent)
    : QStyledItemDelegate(parent)
{
    auto *view = qobject_cast<QAbstractItemView *>(parent);
    if (!view)
        return;

    QAbstractItemModel *model = view->model();
    if (!model)
        return;

    connect(model, &QAbstractItemModel::dataChanged,
            this, [this](const QModelIndex &tl, const QModelIndex &br)
            {
    for (int r = tl.row(); r <= br.row(); ++r) {
        QPersistentModelIndex idx = tl.sibling(r, tl.column());
        docCache.remove(idx);

        // remove all widths for this index
        for (auto it = heightCache.begin(); it != heightCache.end(); ) {
            if (it.key().first == idx)
                it = heightCache.erase(it);
            else
                ++it;
        }
    } });

    connect(model, &QAbstractItemModel::modelReset,
            this, [this]()
            {
    docCache.clear();
    heightCache.clear(); });
}

void RichTextDelegate::paint(QPainter *painter, const QStyleOptionViewItem &option, const QModelIndex &index) const
{
    QStyleOptionViewItem opt(option);
    initStyleOption(&opt, index);

    // if off screen don't draw!!! wasteful,,,
    const QListView *view = qobject_cast<const QListView *>(opt.widget);
    if (view && !opt.rect.intersects(view->viewport()->rect()))
        return;

    const QWidget *cwidget = opt.widget;
    QWidget *widget = const_cast<QWidget *>(cwidget);
    QStyle *style = widget ? widget->style() : QApplication::style();

    // draw the standard item background/selection
    style->drawPrimitive(QStyle::PE_PanelItemViewItem, &opt, painter, widget);

    int textWidth = opt.rect.width();
    if (textWidth <= 0)
    {
        if (view)
            textWidth = view->viewport()->width();
        else if (widget)
            textWidth = widget->width();
    }

    QTextDocument *doc = documentFor(index, textWidth);
    doc->setHtml(index.data(Qt::DisplayRole).toString());

    const int padding = 8;
    textWidth = std::max(1, textWidth - padding);
    doc->setTextWidth(textWidth);

    // draw text within the rect minus padding
    QRect textRect = opt.rect.adjusted(4, 4, -4, -4);
    painter->save();
    painter->setClipRect(opt.rect, Qt::IntersectClip);
    painter->translate(textRect.topLeft());
    QRectF exposed(
        0,
        opt.rect.top() - textRect.top(),
        textWidth,
        opt.rect.height());
    doc->drawContents(painter, exposed);
    painter->restore();
}

QSize RichTextDelegate::sizeHint(
    const QStyleOptionViewItem &option,
    const QModelIndex &index) const
{
    constexpr int Padding = 8;

    int width = option.rect.width();
    const QListView *view =
        qobject_cast<const QListView *>(option.widget);

    if (width <= 0)
    {
        if (view)
            width = view->viewport()->width();
        else if (option.widget)
            width = option.widget->width();
    }

    width = std::max(1, width - Padding * 2);

    QPersistentModelIndex pIndex(index);
    auto key = qMakePair(pIndex, width);

    // use cache for height
    if (auto it = heightCache.find(key); it != heightCache.end())
    {
        return QSize(width + Padding * 2, *it);
    }

    // if there is nothing in cache, use doc and then add to cache
    QTextDocument *doc = documentFor(index, width);
    int height = int(std::ceil(doc->size().height())) + Padding * 2;

    heightCache.insert(key, height);

    return QSize(width + Padding * 2, height);
}

QTextDocument *RichTextDelegate::documentFor(
    const QModelIndex &index, int width) const
{
    auto &entry = docCache[index];

    if (!entry.doc)
    {
        entry.doc = QSharedPointer<QTextDocument>::create();
        entry.doc->setUndoRedoEnabled(false);
        entry.doc->setDocumentMargin(0);
        entry.doc->setHtml(index.data(Qt::DisplayRole).toString());
        entry.width = -1;
    }

    if (entry.width != width)
    {
        entry.doc->setTextWidth(width);
        entry.width = width;
    }

    return entry.doc.data();
}

bool RichTextDelegate::eventFilter(QObject *obj, QEvent *event)
{
    if (event->type() == QEvent::Resize) {
        heightCache.clear();
    }
    return QObject::eventFilter(obj, event);
}
