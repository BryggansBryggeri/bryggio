# RUSTBEER

## Control

Standard is to use a simple PID controller. This is slightly nihilistic given the almost complete model information one usually has when dealing with a simple brewery.
The arguments in favour of PID control are: ignorance, efficiency, laziness; so what if we disregard some informatio? The system is slow and simple and it works fine.

Rustbeer takes the opposite approach: Given a simple and slow system, we have the possibility to have a complicated control. Furthermore, a slow system demands the best possible control in order to save precious time.

The idea is that the inherent inertia in the objective (heating a lot of water) we can have quite long time between signal updates, providing time to calculate a nearly optimal control sequence.

### Literature
http://www.iiisci.org/journal/CV$/sci/pdfs/ZA191KB18.pdf

http://kchbi.chtf.stuba.sk/upload_new/file/Miro/Proc%20problemy%20odovzdane%20zadania/%C5%A0uhaj/Predictive%20modelling%20of%20brewing%20fermentation%20from%20knowledge-based%20to%20black-box%20models.pdf

https://scholarworks.uni.edu/cgi/viewcontent.cgi?article=1661&context=etd
