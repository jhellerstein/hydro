const WebSocket = require('ws');

const wss = new WebSocket.Server({ port: 3001 });

console.log('Test WebSocket server started on ws://localhost:3001');

wss.on('connection', function connection(ws) {
    console.log('Client connected');
    
    ws.on('message', function incoming(message) {
        console.log('Received:', message.toString());
        
        // Echo the message back to test
        ws.send(`Echo: ${message}`);
        
        // Broadcast to all clients
        wss.clients.forEach(function each(client) {
            if (client !== ws && client.readyState === WebSocket.OPEN) {
                client.send(`Broadcast: ${message}`);
            }
        });
    });
    
    ws.on('error', function(error) {
        console.log('WebSocket error:', error);
    });
    
    ws.on('close', function(code, reason) {
        console.log('Client disconnected with code:', code, 'reason:', reason.toString());
    });
    
    // Send welcome message
    ws.send('Welcome to test WebSocket server!');
});
