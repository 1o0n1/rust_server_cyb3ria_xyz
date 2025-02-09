// /var/www/rust_server_cyb3ria_xyz/static/js/menu.js

document.addEventListener('DOMContentLoaded', () => {
    const menuContainer = document.getElementById('menu-container');

    if (menuContainer) {
        fetch('/static/menu.html')
            .then(response => response.text())
            .then(menuHtml => {
                menuContainer.innerHTML = menuHtml;

                //  Этот код теперь ВЫПОЛНЯЕТСЯ ВСЕГДА, когда загружается menu.js:
                const menuToggle = document.getElementById('menu-toggle');
                const mainMenu = document.querySelector('.main-menu');

                if (menuToggle && mainMenu) {
                    menuToggle.addEventListener('click', () => {
                        mainMenu.classList.toggle('collapsed');
                        menuToggle.innerHTML = mainMenu.classList.contains('collapsed') ? '<' : '>';
                    });
                }

                 //  Код для logoutBtn тоже должен быть здесь,
                 //  чтобы обработчик добавлялся на каждой странице:
                const logoutBtn = document.getElementById('logoutBtn');
                if (logoutBtn) {
                   logoutBtn.addEventListener('click', function() {
                       fetch('/api/logout', {
                           method: 'POST',
                           headers: {
                               'Content-Type': 'application/json'
                           },
                       })
                       .then(response => {
                           if (response.ok) {
                               window.location.href = '/static/choice.html';
                           } else {
                               alert('Logout failed.');
                           }
                       })
                       .catch(error => {
                           console.error('Error:', error);
                           alert('Logout failed.');
                       });
                   });
               }
            })
            .catch(error => {
                console.error('Error loading menu:', error);
            });
    }
});