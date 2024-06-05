%*
#schedule(ID,S,T,O,D).

Describes the schedule for an agent.

#parameters
   ID: identifier of the agent
   S: Start position in the form (X,Y) (Flatland style)
   T: Target position in the form (X,Y) (Flatland style)
   O: Initial orientation of the agent
   D: Earliest departure of the agent
*%


schedule(0,(1,0),(0,2),3,0).





parameters(1,2,3).


%*
#parameters(A,B,C).
Some random parameters
#parameters
A:a
B:b
C:c
#parameters
A:a
B:b
C:c
*%


lel() :- parameters()



%*#azad(A,B,C).*%


























% tolle beschreibung
1{pred_direction(ID,D,T):cell(P,O,D,U)}1 :- schedule(ID,P,_,O,T), irgendwas(1,2,3).

some_rule(ID,D,T) :- pred_direction(ID,D,T), cell(P,O,D), schedule((), (), (), (), ()).


%*
#pred_direction(ID,D,T).

#parameters
ID : lel
D: a
T: a
*%