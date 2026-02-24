# Trust Test

Our AI matchmaking system matches interns to projects. We want to test to what extent users tend to rubber-stamp AI recommended matches.

Experimental design questions to answer:

1. should all the students that we're testing receive the same data? or should each one be unique?
2. by how much should we poison the poisoned data?
3. what's our hypothesis?
4. how do we explain what we're doing to the people we're testing in such a way that they behave normally? We don't want them to be *too* critical of the output.

Here's what I'm thinking we'll do, so far.

- Each student will receive the same xlsx file and a unique ID number. It has the top 3 recommended matches and the 0-1 confidence value for the match
- the student will mark each match as approved or denied, and if denied, explain their reasoning in a free text response column
- they will save the xlsx file with their ID number as the filename and upload the file to a onedrive folder (can you grant people write but not read access? Maybe there's some other mechanism we can use)
- we take it from there.
