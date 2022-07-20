# UOverse Server Development Notes

## Movement

Movement works by sending move commands with a direction, and whether the controlled mobile is running or not. A naive implementation would rely upon the client to adhere to timing requirements for movement, and honor each command on the server with an immediate change of position/direction.

Such an implementation would be fairly subject to lag however, if the client is not sending commands faster than the specified game rate. So instead clients are allowed to send movement commands before previous (successful) commands would have completed based on the movement rate, but those commands may not have an immediate effect.

Both of these designs open up the potential for a client to send commands at a rate that exceeds the game rules for movement, but in a way that is unlimited to the extent that it potentially constitutes cheating.

### Excess movement rate prevention

The official UO network protocol supports a design intended to address the above mentioned cheating vulnerability. This is called the "Fastwalk" system in both the ModernUO server and ClassicUO client projects.

The design involves the server issuing 32-bit tickets to authorize walk commands. The client sends one of the tickets it has been issued with each walk command, and the server authenticates the ticket in some fashion. To allow for pre-queuing walk commands as lag compensation, the client stores a buffer of six tickets (which is filled at certain points, e.g. entering the world). The server can thus limit excess movement commands by limiting the maximum rate for ticket issuance.

An alternative approach, which may be in use with official servers now and is supported by the ModernUO server currently, is an entirely server-side rate limiting mechanism. The server keeps track of the number of "queued" movement commands (those received by a client but not yet completed) and rejects any commands which would exceed a defined limit on that number.

The second approach seems like the more straight-forward thing for us to use. There is another design, perhaps akin to what is used in 3D multiplayer games with movement speed hack prevention, and that is to track the total distance covered over a period of time. Knowing how far the player has moved, regardless of the set of movement commands used to achieve it, could still allow for proper rate limiting.
