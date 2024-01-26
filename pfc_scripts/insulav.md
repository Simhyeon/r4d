```
$stream(insulav)
% Source text
\*
$assert(true,$istype(uint,0))
$assert(false,$istype(uint,-1))
$assert(true,$istype(int,0))
*\
$consume|()

% ----
% Technically following process does...
% 1. Isolate lines vertically
% 2. Detect arguments line and rotate around comma. And pad "+"
% 3. Merge into lines with the basis line that matches $assert
% 4. Add some extra spaces in between special characters 
% 5. Align by first ',' and then second ',' and finally by first ')'
% ----
$define(pad_plus,a_n=$pad(c,2,+,$a_n()))
$define(sq,a_ln a_lc=$ifelse($eval($a_ln() %  7 == 5),$enl()
$mapn(pad_plus,$rotatel($comma*(),c,$a_lc())),$a_lc()))
$forline|-($sq($a_LN(),$:()))
$foldreg*|-(\$assert)
$forline*|-($insulah($:()))
$alignby-(1\,2\,1\))
===
# Ran with rad --comment start --silent
$assert( true , $istype( +0, uint ) )
$assert( false, $istype( -1, uint ) )
$assert( true , $istype( +0, int  ) )
```
