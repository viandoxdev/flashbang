package dev.vndx.flashbang.domain

interface Item {
    val itemName: String
    val parentItems: List<Item>
    val childItems: List<Item>
    val leafItems: List<Item>
}