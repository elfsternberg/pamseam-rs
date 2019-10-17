Hey, Elf, if during the calculation of the seams you recorded the max &
min columns of the path, you would automatically have a record of the
column that path meandered *and* could easily compare all the
neighboring columns for *their* intersection, thus creating the
"invalidated" column automagically.  This trades time for space, but it
might be *cheap* space, as you're only recording this data for the
*last* row, if you're doing this in an OO style.

Since the colum meanders, you might have to shuffle the contents of the
invalidation vec.  How do you do that in an OO style?  (Well, you make
the invalidation vec read-only for the scan, then write a new
invalidation rec based on the current pass, dummy.)

On the other hand, if you're doing this with *threads* you now need to
contemplate what it means to record that information. - CF

----

CF, DO NOT COMMIT UNTIL THE TESTS HAVE BEEN RUN.

---

Right.  Gotcha.  Sorry.
