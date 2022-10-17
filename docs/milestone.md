# Confession about version 1.0 and 2.0

R4d was my first project to be this big and consistent. Others were simply a
toy project or simply not this big enough. I was not experienced about semantic
versioning or project maintenance. Frankly speaking I pushed 1.0 too early. I
didn't like the idea of sticking to pre 1.0 stage for ages and believed I
should give definite signs about project progress. After some time passed and I
had regretted my choice but I also learnt semantic versioning and couldn't dare
to make everything break. Although I know most of the crates.io downloads are
bots and "real" person who has even touched my programs are handful, I didn't
want to break the custom anyway.

Therefore 2.0's milestone was to fix bugs and bad decisions that I made.
Technically 2.0 was a real "1.0" stable version. On other other hand 2.0's fix
and updates were mostly centered on my usage which was dynamic text generation
for presentations.

# 3.0 for generic text manipulation

3.0 was a huge goal. Because unlike previous versions, 3.0's goal was to make a
r4d generic text manipulation tool. R4d 2.0 focused on text generation. While I
was adding new macros for r4d, I realized that r4d can be a fully featured text
manipulation engine, which is hella cool. I had to put many breaking changes
and literally hundreds of new macros.

To sum it up, pevious versions of r4d aimed to be a good tool for other
programs thus lacked many features. For 3.0, r4d became a program that can
handle multiple operations by itself.

Due to the nature of its changes, 3.0 version has many breaking changes that is
not compatible with 2.0 macro codes, which is a shame. This will not happen in
the future and will be only permitted by major users' request or critical
security demands.
