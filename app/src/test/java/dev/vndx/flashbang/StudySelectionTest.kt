package dev.vndx.flashbang

import dev.vndx.flashbang.domain.Card
import dev.vndx.flashbang.domain.Study
import dev.vndx.flashbang.domain.Tag
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class StudySelectionTest {

    private fun createTag(name: String, parent: Tag? = null): Tag {
        val ancestors = parent?.let { it.ancestors + it } ?: emptyList()
        val fullPath = if (parent != null) "${parent.fullPath}.$name" else name
        return Tag(fullPath, ancestors)
    }

    private fun createCard(id: String, locations: List<Tag>): Card {
        val card = Card(id, id, locations, "", "", null)
        locations.forEach { tag ->
            tag.addCard(card)
            // Manually populate indirectCards for ancestors to simulate loaded state
            var current: Tag? = tag
            while (current != null) {
                current.addCardIndirect(card)
                current = current.parent
            }
        }
        return card
    }

    @Test
    fun `test single directory fully selected`() {
        val root = createTag("root")
        val card1 = createCard("c1", listOf(root))

        val summary = Study.buildSelectionSummary(setOf(card1))

        assertEquals(1, summary.size)
        assertEquals(root, summary[0])
    }

    @Test
    fun `test parent directory fully selected`() {
        val root = createTag("root")
        val dirA = createTag("A", root)
        val dirB = createTag("B", root)

        val card1 = createCard("c1", listOf(dirA))
        val card2 = createCard("c2", listOf(dirB))

        val summary = Study.buildSelectionSummary(setOf(card1, card2))

        // Should select root because it covers both and is fully selected
        assertEquals(1, summary.size)
        assertEquals(root, summary[0])
    }

    @Test
    fun `test partial parent selection`() {
        val root = createTag("root")
        val dirA = createTag("A", root)
        val dirB = createTag("B", root)

        val card1 = createCard("c1", listOf(dirA))
        val card2 = createCard("c2", listOf(dirB))

        // Select only card1
        val summary = Study.buildSelectionSummary(setOf(card1))

        // Should select dirA, not root (root has unselected card2)
        assertEquals(1, summary.size)
        assertEquals(dirA, summary[0])
    }

    @Test
    fun `test prefer bigger directory`() {
        val root = createTag("root")
        // Add unselected card to root so root itself is not selected
        val unselectedDir = createTag("Unselected", root)
        createCard("unselected", listOf(unselectedDir))

        val dirSmall = createTag("Small", root)
        val dirBig = createTag("Big", root)

        // Card1 is in both Small and Big
        // Big has 9 other cards
        val card1 = createCard("c1", listOf(dirSmall, dirBig))
        val others = (2..10).map { createCard("c$it", listOf(dirBig)) }

        val allBigCards = others + card1

        // Select all cards in Big (which implies Card1 is selected)
        val summary = Study.buildSelectionSummary(allBigCards.toSet())

        // Should select Big.
        // Small is also fully selected (contains Card1).
        // But Big is bigger (size 10 vs 1).
        assertEquals(1, summary.size)
        assertEquals(dirBig, summary[0])
    }

    @Test
    fun `test multiple disjoint directories`() {
        val root = createTag("root")
        val dirA = createTag("A", root)
        val dirB = createTag("B", root)

        val card1 = createCard("c1", listOf(dirA))
        val card2 = createCard("c2", listOf(dirB))
        val card3 = createCard("c3", listOf(dirB)) // Unselected

        // Select c1 and c2. c3 unselected.
        // DirA is fully selected.
        // DirB is partially selected (c2 selected, c3 not).
        // Root is partially selected.

        // Expected: DirA and Card2 (as individual card since DirB not full)

        val summary = Study.buildSelectionSummary(setOf(card1, card2))

        assertEquals(2, summary.size)
        assertTrue(summary.contains(dirA))
        assertTrue(summary.contains(card2))
    }
}
