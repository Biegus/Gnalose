
# Gnalose
## How

Gnalose is esoteric programming language that gets executed starting from the bottom.
The language design makes the code look like it would execute from the top normally when you look the first time.  
It's interpreted language and the intepreter is written in c#.  
Most of the command do the exact opposite of what they look like. There's only one variable type, integer. It can output number or char based on ASCII code.
The name is  "esolang" but revesed.

## OP Codes

``undefine a``   -> defines A (every variable has to be undefined at the end using ``define``)  
 ``define a`` -> undefines A  
``print a`` -> reads from input to a  
``read to a`` ->prints a to output    
``add a to b`` -> subtracts a from every variable but not from b and and from a (a can be immediate value)    
 ``sub a from b``-> adds a to every variable but not to b and and to a (a can be immediate value)  
  ``div a by b``-> mults every variable by a but not b and and not a (a can be immediate value)  
``mult a times b`` divs every variable by a but not b and not a( a can be immediate value)  
``fi`` ->defines beginning of if (look at if section)  
``unmark loop`` makes loop label, every label has to unmarked with ``mark``  
``forget`` pins label to use with ``halt``   
``halt`` ->goes to to mark pinned with ``forget`` (look at goto section)  
``read as number to a`` -> prints value of a as ascii   
``mark loop`` unmarks loop  
``if a greater than b`` -> if a<=b   
``if a not equal to b``-> if a=b    
``if a lower than b`` -> if a>=b   
``if a equal to b`` -> if a!=b   
``if a lower or equal than b`` -> if a>b  
``if a greater or equal than b`` -> if a<b  
``undefine single a[3]`` defines 3 elements array, array has to be undefined with ``define single``  
 ``define single a`` ->undefines array  a


comments: ``comment/INSTRUCTIONS`` for instance ``hey ssup/undefine a``
## If
``if`` is written in reverse so you first write ``fi`` and end with conditional ( tho the if will behave as it was the other way around,  it will check conditional at ``fi``)
```

if a=10 which is the case/if a is not equal to 10
this will always happen/sub 999 from b
fi
sets a=10/sub 10 from b  
undefine b  
undefine a
```

## Goto 
You can only jump to already defined marks. You must visit mark first for him to be defined. Also instead of giving an argument to ``halt`` you write ``forget`` with the place name
```
goto/halt
set goto arguments to place/forget place
some code here/
unmark place
```
you do not need to unmark place with ``mark`` because this program is never gonna finish execution anyway

## Arrays
after you define array with ``undefine single NAME[AMOUNT]`` you can use it as a normal variable. The index may be immediate value or suplied by variable. The index cannot be define by element of another array.
Remember that you have undefine you array with ``define single NAME`` when you not longer using it  
``add 3 to b[3]``    
``add 3 to b[c]``   
~~``add 3 to b[d[3]]``~~

## A simple program that will crash
```
undefine a
```
reason: you have to undefine a (never heard about memory leaks?)

## A simple program that will not crash
```
define a
undefine a
```


