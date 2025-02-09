document.addEventListener('DOMContentLoaded', () => {
    const menuContainer = document.getElementById('menu-container');
    
    if (menuContainer) {
        fetch('/static/menu.html')
            .then(response => response.text())
            .then(menuHtml => {
                menuContainer.innerHTML = menuHtml;
    
                const menuToggle = document.getElementById('menu-toggle');
                const mainMenu = document.querySelector('.main-menu');
    
                if (!menuToggle || !mainMenu) return;
    
                function updateButtonPosition() {
                    if (window.innerWidth >= 769) {
                        menuToggle.style.left = mainMenu.classList.contains('active') ? '10px' : '260px';
                    } else {
                        menuToggle.style.left = '10px';
                    }
                }
    
                // Устанавливаем начальное состояние кнопки
                updateButtonPosition();
                
                if (window.innerWidth >= 769) {
                    menuToggle.innerHTML = '<';
                } else {
                    menuToggle.innerHTML = '⋯';
                }
    
                menuToggle.addEventListener('click', () => {
                    mainMenu.classList.toggle('active');
                    updateButtonPosition();
    
                    if (window.innerWidth >= 769) {
                        menuToggle.innerHTML = mainMenu.classList.contains('active') ? '>' : '<';
                    } else {
                        menuToggle.innerHTML = mainMenu.classList.contains('active') ? '✕' : '⋯';
                    }
                });
    
                window.addEventListener('resize', updateButtonPosition);
    
                const logoutBtn = document.getElementById('logoutBtn');
                if (logoutBtn) {
                    logoutBtn.addEventListener('click', function() {
                        fetch('/api/logout', {
                            method: 'POST',
                            headers: { 'Content-Type': 'application/json' },
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
