
 - MMO
 - browser game: 3D with wgpu, UI with HTML
 - hard sci-fi
 - colonization: players can colonize the galaxy
 - production: players need to gather and process various materials
 - diplomacy: players manage factions that have complex diplomatic relations

# Galaxy

## Overview

 - galaxy consists of star systems
 - star systems consist of bodies
 - bodies are planets, moons, stars
 - star systems and bodies are managed as a tree:
  - stores orbital elements: initial position, period, etc.[2]
  - root node: center of galaxy
  - 1st level: star system (bary centers)
  - 2nd level: stars (possibly in a tree for binaries etc. [1])
  - 2nd level: planets (or bary center for planet binaries, e.g. Pluto an Charon)
  - 3rd level: moons
 - efficient lookup of stars (objects) in a region of space: octtree with LOD.
  - this needs to be maintained as objects move
  - we will only track star systems in this data structure

## Parameters

Some parameters from the real world:

 - Milky Way [3]:
  - 100 - 400 billion stars (~1 billion mapped by Gaia)
  - diameter: 26.8 ± 1.1 kpc, 87400 ± 3600 lyr
  - sun's galactic period: 212 Myr
   - if 1 year in-game takes 1 sec, the sun would take 6.7 realtime years for a revolution

## Dynamic vs Static

It seems we can either:
 1. make stars in the galaxy static and star systems dynamic
  - slow time scale
  - easy galaxy-scale octtree
  - regions can be procedularly generated just in time
 2. make stars in the galaxy dynamic and star systems static
  - fast time scale
  - need to maintain the galaxy-wide octtree
  - no JIT procgen
  - star systems can be very abstract, saving computational resources
 3. make both dynamic
  - slow time scale, since we can't have the planets move too fast
  - stars move very slowly
  - need to maintain the galaxy-wide octtree
  - no JIT procgen

## Time Scale

Time scale depends on the dynamic vs static systems in the previous section.
But it is also inter-dependent with spaceship speeds. We prefer slower than light speeds.

We could introduce late-game tech like warp drives or worm holes to allow faster travel.
We think we don't want to use any FTL tech, even warp drives. Worm holes are allowed, but must be transported classically.
We could seed the galaxy with "ancient" worm holes to make it easier to colonize the whole galaxy. Although we might not want this, given the
computational resources needed. And given the sheer number of star systems, it is not really needed.

The following table uses 0.5c as travel speed and assumes 1 billion star systems in the galaxy.

| Time scale | Galaxy crossing                    | Proxima Centuri  | Star systems in 24h | 100k years                        |
|------------|------------------------------------|------------------|---------------------|-----------------------------------|
| 1s         | 2days 33m 20s                      | 8s 493ms         | 3908990464          | 1day 3h 46m 40s                   |
| 3m         | 11months 29days 7h 50m 24s         | 25m 28s 740ms    | 120647              | 6months 25days 16h 38m 24s        |
| 5m         | 1year 7months 28days 14h 44m 48s   | 42m 27s 900ms    | 43433               | 11months 12days 9h 10m 24s        |
| 10m        | 3years 3months 26days 19h 39m 12s  | 1h 24m 55s 800ms | 10858               | 1year 10months 24days 19h 4m      |
| 15m        | 4years 11months 24days 23h 50m 24s | 2h 7m 23s 700ms  | 4825                | 2years 10months 6days 18h 24m     |
| 30m        | 9years 11months 19days 13h 50m 24s | 4h 14m 47s 400ms | 1206                | 5years 8months 13days 13h 31m 12s |
| 1h         | 19years 11months 8days 17h 50m 24s | 8h 29m 34s 800ms | 301                 | 11years 4months 27days 3h 45m 36s |

We think 5 to 15 min (realtime) / yr (in-game) is a good compromise. At this time-scale we can ignore the galactic orbital motions of star systems.

## Conclusion

 - fixed star system positions
 - dynamic star systems
 - time scale: about 10 min / yr


# Production




[1]: https://en.wikipedia.org/wiki/Star_system#Hierarchical_systems
[2]: https://en.wikipedia.org/wiki/Orbital_elements
[3]: https://en.wikipedia.org/wiki/Milky_Way
