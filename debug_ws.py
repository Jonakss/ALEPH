import asyncio
import websockets
import json

async def hello():
    uri = "ws://localhost:3030"
    try:
        async with websockets.connect(uri, ping_interval=None) as websocket:
            print(f"Connected to {uri}")
            
            # Send specific "upgrade" if needed, but pure WS connect should work 
            # based on daemon.rs implementation.
            
            while True:
                msg = await websocket.recv()
                try:
                    data = json.loads(msg)
                    # Handle Rust Enum wrapping if present
                    payload = data.get("Telemetry", data)
                    
                    activity = payload.get("reservoir_activity", [])
                    activations = payload.get("activations", [])
                    
                    print(f"Update: {len(msg)} bytes | Reservoir Active: {len(activity)} (Sparse) | Activations: {len(activations)} nodes")
                    
                    if len(activity) > 0:
                        # Print first few items to verify format
                        print(f"Sample: {activity[:5]}")
                        
                except json.JSONDecodeError:
                    print(f"Received (Raw): {msg[:100]}...")
                    
    except Exception as e:
        print(f"Error: {e}")

if __name__ == "__main__":
    asyncio.run(hello())
