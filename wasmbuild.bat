@ECHO OFF

REM CALL :Stage chapter-01-hellorust
REM CALL :Stage chapter-02-helloecs
REM CALL :Stage chapter-03-walkmap
REM CALL :Stage chapter-04-newmap
REM CALL :Stage chapter-05-fov
REM CALL :Stage chapter-06-monsters
REM CALL :Stage chapter-07-damage
REM CALL :Stage chapter-08-ui
REM CALL :Stage chapter-09-items
REM CALL :Stage chapter-10-ranged
REM CALL :Stage chapter-11-loadsave
REM CALL :Stage chapter-12-delvingdeeper
REM CALL :Stage chapter-13-difficulty
REM CALL :Stage chapter-14-gear
REM CALL :Stage chapter-16-nicewalls
REM CALL :Stage chapter-17-blood
REM CALL :Stage chapter-18-particles
REM CALL :Stage chapter-19-food
REM CALL :Stage chapter-20-magicmapping
REM CALL :Stage chapter-21-rexmenu
REM CALL :Stage chapter-22-simpletraps
REM CALL :Stage chapter-23-generic-map
REM CALL :Stage chapter-24-map-testing
REM CALL :Stage chapter-25-bsproom-dungeons
REM CALL :Stage chapter-26-bsp-interiors
REM CALL :Stage chapter-27-cellular-automata
REM CALL :Stage chapter-28-drunkards-walk
REM CALL :Stage chapter-29-mazes
REM CALL :Stage chapter-30-dla
REM CALL :Stage chapter-31-symmetry
REM CALL :Stage chapter-32-voronoi
REM CALL :Stage chapter-33-wfc
REM CALL :Stage chapter-34-vaults
REM CALL :Stage chapter-35-vaults2
REM CALL :Stage chapter-36-layers
REM CALL :Stage chapter-37-layers2
REM CALL :Stage chapter-38-rooms
REM CALL :Stage chapter-39-halls
REM CALL :Stage chapter-40-doors
REM CALL :Stage chapter-41-camera
REM CALL :Stage chapter-45-raws1
REM CALL :Stage chapter-46-raws2
REM CALL :Stage chapter-47-town1
REM CALL :Stage chapter-48-town2
REM CALL :Stage chapter-49-town3
REM CALL :Stage chapter-50-stats
REM CALL :Stage chapter-51-gear
REM CALL :Stage chapter-52-ui
REM CALL :Stage chapter-53-woods
REM CALL :Stage chapter-54-xp
CALL :Stage chapter-55-backtrack
REM CALL :Stage chapter-56-caverns
REM CALL :Stage chapter-57-ai
REM CALL :Stage chapter-58-itemstats
REM CALL :Stage chapter-59-caverns2
REM CALL :Stage chapter-60-caverns3
REM CALL :Stage chapter-61-townportal
REM CALL :Stage chapter-62-magicitems
REM CALL :Stage chapter-63-effects
REM CALL :Stage chapter-64-curses
REM CALL :Stage chapter-65-items
REM CALL :Stage chapter-66-spells
REM CALL :Stage chapter-67-dragon
REM CALL :Stage chapter-68-mushrooms
REM CALL :Stage chapter-69-mushrooms2
REM CALL :Stage chapter-70-missiles

REM Publish or perish
cd book\book\wasm
pscp -r * herbert@172.16.10.193:/var/www/bfnightly/rustbook/wasm
cd ..\..\..

EXIT /B 0

REM Usage: Stage Chapter
:Stage
cd %~1
cargo build --release --target wasm32-unknown-unknown
if %errorlevel% neq 0 exit /b %errorlevel%
wasm-bindgen ..\target\wasm32-unknown-unknown\release\%~1.wasm --out-dir ../book/book/wasm/%~1 --no-modules --no-typescript
if %errorlevel% neq 0 exit /b %errorlevel%
cd ..
move .\book\book\wasm\%~1\%~1.js .\book\book\wasm\%~1\myblob.js
move .\book\book\wasm\%~1\%~1_bg.wasm ./book/book/wasm/%~1/myblob_bg.wasm
copy index.html .\book\book\wasm\%~1
