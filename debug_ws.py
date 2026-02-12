import socket
import base64
import os

def test_handshake():
    host = "127.0.0.1"
    port = 3030
    
    # Generate random key
    key = base64.b64encode(os.urandom(16)).decode('utf-8')
    print(f"DEBUG: Generated Key: {key}")

    junk_header = "X-Junk: " + ("A" * 2000) + "\r\n"
    request = (
        f"GET / HTTP/1.1\r\n"
        f"Host: {host}:{port}\r\n"
        f"Upgrade: websocket\r\n"
        f"Connection: Upgrade\r\n"
        f"Sec-WebSocket-Key: {key}\r\n"
        f"Sec-WebSocket-Version: 13\r\n"
        f"{junk_header}"
        f"\r\n"
    )

    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    try:
        s.connect((host, port))
        print(f"Connected to {host}:{port}")
        s.sendall(request.encode('utf-8'))
        
        response = s.recv(4096)
        print("\n--- SERVER RESPONSE ---")
        print(response.decode('utf-8', errors='replace'))
        print("-----------------------")
        
    except Exception as e:
        print(f"Error: {e}")
    finally:
        s.close()

if __name__ == "__main__":
    test_handshake()
