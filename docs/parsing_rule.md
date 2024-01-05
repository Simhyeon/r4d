# Concerns

1. I changed greedy behaviour to not strip at all. Is it desirable?

This makes usage much sensible. Yet it makes program logic inconsistent.
Everything has its own pros and cons.

2. I changed fn strip to use Neversplit after previous changes. Is it ok?

    -> I removed strip from log message and log error, because why they?

    -> I added new enum variatnt GreedyStrip

3. Append's syntax is kinda bullshit... I made it "work" but it is hard to use
   anyway. Frankly speaking, trailer was a bad idea.

# NOTE

Greedy doesn't strip literal

Never strip literal but cuts into pieces
