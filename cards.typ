//![FLASHBANG IGNORE]

#let setup(doc) = {
  set text(size: 16pt)
  set page(margin: (x: 2em, y: 1em), height: auto)
  doc
}

#let card(id, name, tags) = {
  v(1em)
  box(
    stroke: (left: 2pt + red),
    inset: 10pt,
    fill: luma(240),
    width: 100%,
  {
    set text(size: 10pt, fill: black, weight: "bold", font: "DejaVu Sans Mono")
    for path in tags {
      for tag in path.split(".") {
        tag
        if not path.ends-with(tag) {
          [ $triangle.filled.small.r$ ]
        }
      }
      linebreak()
    }
    set text(font: "", size: 16pt)
    v(-5pt)
    name
  })
}
#let answer = {
  line(length: 100%, stroke: 1pt + luma(200))
}
