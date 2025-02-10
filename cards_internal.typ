#let setup(doc) = {
  set page(height: auto, width: 400pt, margin: 1em)
  doc
}

#let card(id, name, tags) = {
  pagebreak()
  place(horizon + right, text(size: 30pt, fill: luma(200))[?])
}

#let answer = {
  pagebreak()
  place(horizon + right, text(size: 30pt, fill: luma(200))[!])
}
