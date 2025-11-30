#!/usr/bin/env python3
"""
MCP Client Example for Council Of Dicks

This script demonstrates how to interact with the Council MCP server.
"""

import json
import socket
import sys

class CouncilMcpClient:
    def __init__(self, host='localhost', port=9001):
        self.host = host
        self.port = port
        self.request_id = 0
    
    def connect(self):
        """Connect to MCP server"""
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        try:
            self.sock.connect((self.host, self.port))
            print(f"✅ Connected to {self.host}:{self.port}")
        except ConnectionRefusedError:
            print(f"❌ Connection refused. Is the MCP server running?")
            print(f"   Start the Council app and click 'Start MCP Server'")
            sys.exit(1)
    
    def send_request(self, method, params=None):
        """Send JSON-RPC request to server"""
        self.request_id += 1
        request = {
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": method
        }
        if params:
            request["params"] = params
        
        request_json = json.dumps(request) + "\n"
        print(f"\n📤 Sending: {method}")
        print(f"   {request_json.strip()}")
        
        self.sock.send(request_json.encode())
        
        # Receive response
        response_data = self.sock.recv(4096).decode()
        response = json.loads(response_data.strip())
        
        print(f"\n📥 Response:")
        print(f"   {json.dumps(response, indent=2)}")
        
        return response
    
    def list_tools(self):
        """List available MCP tools"""
        return self.send_request("tools/list")
    
    def ask_question(self, question, wait_for_consensus=False):
        """Ask a question to the council"""
        return self.send_request("council/ask", {
            "question": question,
            "wait_for_consensus": wait_for_consensus
        })
    
    def get_session(self, session_id):
        """Get session details"""
        return self.send_request("council/get_session", {
            "session_id": session_id
        })
    
    def list_sessions(self):
        """List all sessions"""
        return self.send_request("council/list_sessions")
    
    def close(self):
        """Close connection"""
        self.sock.close()
        print("\n👋 Connection closed")

def main():
    print("🏛️  Council Of Dicks - MCP Client Example")
    print("=" * 50)
    
    client = CouncilMcpClient()
    client.connect()
    
    try:
        # 1. List available tools
        print("\n" + "=" * 50)
        print("1️⃣  LIST AVAILABLE TOOLS")
        print("=" * 50)
        tools_response = client.list_tools()
        
        if "result" in tools_response and "tools" in tools_response["result"]:
            print("\n🔧 Available tools:")
            for tool in tools_response["result"]["tools"]:
                print(f"   • {tool['name']}: {tool['description']}")
        
        # 2. Ask a question
        print("\n" + "=" * 50)
        print("2️⃣  ASK A QUESTION")
        print("=" * 50)
        question = "What is the best programming language for systems programming?"
        ask_response = client.ask_question(question)
        
        session_id = None
        if "result" in ask_response and "session_id" in ask_response["result"]:
            session_id = ask_response["result"]["session_id"]
            print(f"\n✅ Session created: {session_id}")
            print(f"   Status: {ask_response['result']['status']}")
        
        # 3. Get session details
        if session_id:
            print("\n" + "=" * 50)
            print("3️⃣  GET SESSION DETAILS")
            print("=" * 50)
            session_response = client.get_session(session_id)
            
            if "result" in session_response:
                session = session_response["result"]
                print(f"\n📋 Session: {session['id']}")
                print(f"   Question: {session['question']}")
                print(f"   Status: {session['status']}")
                print(f"   Responses: {len(session['responses'])}")
                print(f"   Consensus: {session['consensus']}")
        
        # 4. List all sessions
        print("\n" + "=" * 50)
        print("4️⃣  LIST ALL SESSIONS")
        print("=" * 50)
        list_response = client.list_sessions()
        
        if "result" in list_response:
            sessions = list_response["result"]
            print(f"\n📚 Total sessions: {len(sessions)}")
            for i, session in enumerate(sessions, 1):
                print(f"\n   Session {i}:")
                print(f"   • ID: {session['id']}")
                print(f"   • Question: {session['question'][:60]}...")
                print(f"   • Status: {session['status']}")
                print(f"   • Responses: {len(session['responses'])}")
        
        print("\n" + "=" * 50)
        print("✅ Demo completed successfully!")
        print("=" * 50)
        
    except Exception as e:
        print(f"\n❌ Error: {e}")
        import traceback
        traceback.print_exc()
    
    finally:
        client.close()

if __name__ == "__main__":
    main()
