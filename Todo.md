7/3

What got done: I have set up the server and i now have the connections. I can allow up to 4 people to connect. Next step is to stream music, the host will only be allowed to and the users can maybe vote to skip

Future: Use rataturi to make it look nice

Issue: does not tell user if there are 4 connectiosn. im being told to use Arc<mutex> for concurrency. but as of now it seems to be doing good

Mutext: Only one at a time, since multiple people could be joining, functions like current connection count good experince an overwrite. Mutex will basically lock that function so it doenst happen, it will force it to tkae turns

Arc: So this allows multiple things to own a data value. Since in rust, there can only be 1 owner at a time having using tokio to do multiple tasks would cause issues so Arc is there to solve that. It basically clonse a value and then another task can use it, both value point to the same spot in memory. Think of a group project on a google doc. (old school engineers are insane)



add entry code stuff



7/5

A huge revamp of the code. I wanted to add the music section to this part but i wasnt sure how. I ended up deciding that the best way to do it is to make the music part a 5th client. But then there was the issue of only allowing 4 people to connect to the server. to solve this I made the music bot its own little place on the server. now it can join without affecting the amount of human users that can be in a server. Doing this did require me to rewrite the most of the code. What i had before was static, how would i prompt the music code to enter a name and a valid room number? The work around i came up with was to have a token that is hard coded in the server class, when the music client connects to the server it will send the token and the server will validate it and allow it to join. This isnt the best practice for security but since this is just a project to get familar with building a server it should work. I also was able to get the group chat feature working so everyone that connects can send and see messages. The next part is to acutal get the music client to stream music and find a way to get rodio working.




7/7

Ive given up on adding the music part. Its just too confusing and ive rewritten my code multiple times, im just sticking with what i got and will make the musicbot a ai chat bot. hopefully that will work instead. im using google gemini for that



7/10

need to test if the ai client is working once i have more tokens. the ui for the client is done. ratatui is a pain but it should be working





7/11

well i can't test the ai chatbot caused i used all my tokens trying to get it to work.  I was using the wrong model and getting errors so that ate all the tokens.