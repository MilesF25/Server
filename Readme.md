

## title
So this is a project i made cause im studying for my comptia sec+ exam and i was learning about databases and servers. The more I learn about cyber security I get more intrgiued on the low level things, like what does the code look like that lets firewalls block certain actions. Being a SOC analyst would mean knowing how to use a firewall but what I want to know is how do they work. So I did that with servers. This most likely poorly put together server was my attempt at understand how servers are built and how it runs.

This project was hard. Diving into network programing with zero experience is not something id recommend. It is very different from regular programing and Trying to find info on how to build a lan server was hard. The hardest part was trying to wrap my head around threading on top of learning about rust features like Arc and Mutex. My original plan for the server was to have the host start the server and then stream music of their choise to all the people that joined. The first road block was not really thinking about the structure of the program. My first idea was the host starting and running the server and managing the server from on console. That ended badly cause I got overwhelmed with the code and trying to figure out how to structre everything gave me a headache so to keep it simple I made it so there is no host and the server is its own thing. 

The next hurdle for me was figuring out how to send audio. I was not able to implement it, it was just too complex. I had to do something so instead of audio I decided to do a chat bot. The chat bot joins the program using its own special token. If the user calls the chatbots name it will respond to them. Even though a part of me is dissapointed that I didnt implement the audio feature, as a first time project of this nature im willing to give myself a pass, this was hard. But now I know how to plan out my server structure better and I can be prepared. Hopefully the next time I do something like this I will be able to have a less rigid server structure, i feel like that is what held me back from implementing audio.

# LAN Server (WIP)

A real-time chat server built in Rust with room-based architecture and music bot integration.

## Features

- **Multi-user chat rooms** with capacity limits (max 4 users per room)
- **Access codes** for secure room entry
- **Chat bot support** with dedicated authentication tokens
- **Async networking** powered by Tokio for handling concurrent connections

## Architecture

### Core Components


#### `serverstruct.rs`
Implements the `Server_Room` struct managing room state:
- **User management**: Track connected users and their socket addresses
- **Capacity enforcement**: Limit rooms to 4 users and 1 music bot
- **Access control**: Entry code validation for room access
- **Bot authentication**: Separate token validation for music bot connections

## Technology Stack

- **Tokio** - Async runtime for concurrent I/O
- **Ratatui** - Terminal UI framework
- **Gemini-rs** - AI integration support
- **Crossterm** - Cross-platform terminal utilities

## How to Run

### Prerequisites

Before running the project, make sure you have:

- Rust and Cargo installed
- A Google AI Studio API key (required for the AI chatbot)

---

### Environment Setup

The AI chatbot requires a Google AI Studio API key.

1. Create a file named `.env` inside the `music/` directory.
2. Add the following line to the file:

```env
MY_API_KEY="your_api_key_here"
```

Replace `your_api_key_here` with your actual Google AI Studio API key.

> **Note:** If you only want to run the server and regular clients, you do not need the `.env` file. It is only required for the AI chatbot.

---

### Running the Application

#### 1. Start the Server

From the project root, run:

```bash
cargo run --bin host
```

Leave this terminal open while the server is running.

---

#### 2. Start One or More Clients

Open a new terminal for each client and run:

```bash
cargo run --bin client
```

Up to **4 clients** can connect to the same chat room simultaneously.

---

#### 3. (Optional) Start the AI Chatbot

If you created the `.env` file, open another terminal and run:

```bash
cargo run --bin ai_chatbot
```

The chatbot will authenticate using the API key stored in the `.env` file and join the chat room.