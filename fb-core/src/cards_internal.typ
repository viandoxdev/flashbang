#set page(height: auto, margin: 1em, fill: none)

#let card(id, name, tags) = {
  pagebreak()
  place(horizon + right, text(size: 30pt, fill: luma(200))[?])
}

#let answer = {
  pagebreak()
  place(horizon + right, text(size: 30pt, fill: luma(200))[!])
}
