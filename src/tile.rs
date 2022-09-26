#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Tile {
    Null, // Should never be present

    Air,
    Grass,
    Dirt,
    Stone,
}

/*
 x     xx    x     xxx  xx x xx
 x xxx    x  x x      x  x
   x          x x xx      x x
x       xxxx    x  xx xx    x
x  xx xx          xx   xx x x
x         xx  xx     x      x  x
  x    xx  xx xx x  x  x   x  x
xx          x              x   x
 x x   x     x        x   x
x  x  xx   x          x x
  x x   x x        xx  x       x
  x     x   xx   x   x  x
 x            x     xx         x
x   x       xxx  x     x xxxx  x
 x   x   xx x  xx  x  xx  x  x
     xxx x x   x  x     x  x x
  x   x x   x  xx   x    x  xx
  x  x  x          xx xx  x x x
xx x      xx     x x
 x x          x x        xx   xx
 x  xx x xxx     xx        x
xx       x x    xx x x x     x x
x     x      x    x      x  x
      x             x x       x
  x  x   x x    x       x     x
                x  x      xxxx
    x  x   x x       xx  x
x x x  x     x    xx x x x
xx     x    x x   xx   x      x
  x   xxxx  x x  xx x  x    x
x        x     x        x x  x x
   x xx        x x  x  x  xxx */
