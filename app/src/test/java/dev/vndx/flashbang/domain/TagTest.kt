package dev.vndx.flashbang.domain

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class TagTest {

    private fun createTag(fullPath: String, ancestors: List<Tag> = emptyList()): Tag {
        return Tag(fullPath, ancestors)
    }

    private fun createCard(id: String): Card {
        return Card(id, "Card $id", emptyList(), "Q", "A", null)
    }

    @Test
    fun `test tag name derivation`() {
        val root = createTag("root")
        assertEquals("root", root.name)

        val child = createTag("root.child")
        assertEquals("child", child.name)
    }

    @Test
    fun `test parent and root derivation`() {
        val root = createTag("root")
        assertNull(root.parent)
        assertEquals(root, root.root)

        val child = Tag("root.child", listOf(root))
        assertEquals(root, child.parent)
        assertEquals(root, child.root)

        val grandChild = Tag("root.child.grand", listOf(root, child))
        assertEquals(child, grandChild.parent)
        assertEquals(root, grandChild.root)
    }

    @Test
    fun `test addChild`() {
        val parent = createTag("parent")
        val child = createTag("parent.child")

        parent.addChild(child)

        assertTrue(parent.children.contains(child))
        assertEquals(1, parent.children.size)
    }

    @Test
    fun `test addCard`() {
        val tag = createTag("tag")
        val card = createCard("c1")

        tag.addCard(card)

        assertTrue(tag.cards.contains(card))
        assertTrue(tag.indirectCards.contains(card))
        assertEquals(1, tag.cards.size)
        assertEquals(1, tag.indirectCards.size)
    }

    @Test
    fun `test addCard with duplicates`() {
        val tag = createTag("tag")
        val card = createCard("c1")

        tag.addCard(card)
        tag.addCard(card)

        assertEquals(1, tag.cards.size)
        assertEquals(1, tag.indirectCards.size)
    }

    @Test
    fun `test addCardIndirect`() {
        val tag = createTag("tag")
        val card = createCard("c1")

        tag.addCardIndirect(card)

        assertTrue(tag.indirectCards.contains(card))
        assertTrue(tag.cards.isEmpty())
        assertEquals(1, tag.indirectCards.size)
    }

    @Test
    fun `test tag initialization adds itself to parent`() {
        val root = createTag("root")
        val child = Tag("root.child", listOf(root))

        assertTrue(root.children.contains(child))
    }

    @Test
    fun `test Item interface implementation`() {
        val root = createTag("root")
        val child = Tag("root.child", listOf(root))
        val card = createCard("c1")
        root.addCard(card)

        assertEquals("root", root.itemName)
        // root's childItems should be children (child) + cards (card)
        val children = root.childItems
        assertEquals(2, children.size)
        assertTrue(children.contains(child))
        assertTrue(children.contains(card))

        // leafItems should be indirectCards
        assertEquals(1, root.leafItems.size)
        assertTrue(root.leafItems.contains(card))

        // parentItems should be parent
        assertTrue(root.parentItems.isEmpty())
        assertEquals(listOf(root), child.parentItems)
    }
}
