package dev.vndx.flashbang.domain

class Tag(
    val fullPath: String,
    val ancestors: List<Tag>,
) {
    init {
        ancestors.lastOrNull()?.addChild(this)
    }

    val name = fullPath.split(".").last()
    val children = mutableSetOf<Tag>()
        private set
    val cards = mutableSetOf<Card>()
        private set
    val indirectCards = mutableSetOf<Card>()
        private set

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
}