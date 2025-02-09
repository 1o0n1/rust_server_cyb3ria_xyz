// /var/www/rust_server_cyb3ria_xyz/static/js/scripts.js

// Получаем IP-адрес клиента
let ipAddress = '';  // Объявляем переменную в начале файла
fetch('https://api.ipify.org?format=json')
    .then(response => response.json())
    .then(data => {
        ipAddress = data.ip;
        console.log('Client IP Address:', ipAddress);
    })
    .catch(error => {
        console.error('Error fetching IP address:', error);
    });

// MAC-адрес (заглушка)
let macAddress = '00:00:00:00:00:00';

let ws = null; // Глобальная переменная для WebSocket

function connectWebSocket() {
    if (ws) {
        ws.close();
        console.log('WebSocket connection closed');
    }

    const sessionId = localStorage.getItem('session_id');
    if (!sessionId) {
        console.error('Session ID not found.');
        return;
    }

    ws = new WebSocket(`wss://cyb3ria.xyz/api/ws?session_id=${encodeURIComponent(sessionId)}`);

    ws.onopen = () => {
        console.log('WebSocket connection established');
        document.getElementById('connection-status').textContent = "Connected";
    };

    ws.onmessage = event => {
        const li = document.createElement('li');
        li.textContent = event.data;
        if (messages) { // Проверяем, существует ли messages
            messages.appendChild(li);
             messages.scrollTop = messages.scrollHeight;
        }

    };

    ws.onerror = error => {
        console.error('WebSocket error:', error);
        document.getElementById('connection-status').textContent = "Error";
        setTimeout(connectWebSocket, 5000);
    };

    ws.onclose = () => {
        console.log('WebSocket connection closed');
        document.getElementById('connection-status').textContent = "Disconnected";
        setTimeout(connectWebSocket, 5000);
    };
}

//  DOMContentLoaded для chat.html
document.addEventListener('DOMContentLoaded', () => {
     const messages = document.getElementById('messages');
     const form = document.getElementById('form');
     const input = document.getElementById('name');

     if (messages && form && input) { // Проверка на null
        connectWebSocket(); // Подключаем WebSocket только на странице чата
        form.addEventListener('submit', event => {
          event.preventDefault();
          const message = {
              message: input.value,
              ip: ipAddress,
              mac: macAddress
           };
            if(ws){ // Проверяем, что ws определен
                ws.send(JSON.stringify(message));
            }

            input.value = '';
        });
    }
});