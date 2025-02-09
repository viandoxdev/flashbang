#import "cards.typ": *

#show: setup

#card("Taylor-Langrange", ("Analyse.Dérivation", "Analyse.Taylor"))

Théorème de Taylor-Lagrange, et conditions d'application.

#answer

Soit $f : [a, b] -> RR$, $C^n$ sur $[a, b]$ et $D^(n+1)$ sur $]a,b[$

Il existe $c in ]a, b[$ tel que
$
  f(b) = sum_(k = 0)^(n) f^((k))(a) (x - a)^k / (k!) + f^(n+1) (c) (x - a)^(n+1) / ((n+1)!)
$

#card("Taylor reste intégrale", ("Analyse.Intégration", "Analyse.Taylor"))

Théorème de Taylor reste intégrale, et conditions d'application.

#answer

Soit $f : [a, b] -> RR$, $C^(n+1)$

$
  f(b) = sum_(k = 0)^(n) f^((k))(a) (x - a)^k / (k!) + integral_a^b f^((n + 1)) (t) (b - t)^n / (n!) dif t
$

#card("Test", ("Test",))

This is a test card

#answer

#lorem(100)

#card("Filler", ("A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z"))

$
  ~^A_B
$

#answer

vroooom
