package dev.vndx.flashbang.domain

class Tag(
    val fullPath: String,
    val ancestors: List<Tag>,
) : Item {
    init {
        ancestors.lastOrNull()?.addChild(this)
    }

    val name = fullPath.split(".").last()
    val children = mutableSetOf<Tag>()
    val cards = mutableSetOf<Card>()
    val indirectCards = mutableSetOf<Card>()

    val parent get() = ancestors.lastOrNull()
    val root get() = ancestors.firstOrNull() ?: this

    fun addChild(child: Tag) {
        children.add(child)
    }

    fun addCard(card: Card) {
        cards.add(card)
        indirectCards.add(card)
    }

    fun addCardIndirect(card: Card) {
        indirectCards.add(card)
    }

    override val itemName: String
        get() = name
    override val childItems: List<Item>
        get() = children.toList() + cards.toList()
    override val leafItems: List<Item>
        get() = indirectCards.toList()
    override val parentItems: List<Item>
        get() = listOfNotNull(parent)
}