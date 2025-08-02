// Copy to clipboard functionality
function copyToClipboard(button) {
    const codeElement = button.parentElement.querySelector('code');
    const text = codeElement.textContent;
    
    navigator.clipboard.writeText(text).then(() => {
        // Change button text temporarily
        const originalHTML = button.innerHTML;
        button.innerHTML = '<i class="fas fa-check"></i>';
        button.style.background = '#27ca3f';
        
        setTimeout(() => {
            button.innerHTML = originalHTML;
            button.style.background = '#2d2d2d';
        }, 2000);
    }).catch(err => {
        console.error('Failed to copy text: ', err);
        // Fallback for older browsers
        const textArea = document.createElement('textarea');
        textArea.value = text;
        document.body.appendChild(textArea);
        textArea.select();
        document.execCommand('copy');
        document.body.removeChild(textArea);
        
        // Show feedback
        const originalHTML = button.innerHTML;
        button.innerHTML = '<i class="fas fa-check"></i>';
        button.style.background = '#27ca3f';
        
        setTimeout(() => {
            button.innerHTML = originalHTML;
            button.style.background = '#2d2d2d';
        }, 2000);
    });
}

// Smooth scrolling for navigation links
document.addEventListener('DOMContentLoaded', function() {
    // Smooth scrolling for anchor links
    const links = document.querySelectorAll('a[href^="#"]');
    
    links.forEach(link => {
        link.addEventListener('click', function(e) {
            e.preventDefault();
            
            const targetId = this.getAttribute('href');
            const targetSection = document.querySelector(targetId);
            
            if (targetSection) {
                const offsetTop = targetSection.offsetTop - 80; // Account for fixed navbar
                
                window.scrollTo({
                    top: offsetTop,
                    behavior: 'smooth'
                });
            }
        });
    });
    
    // Navbar background on scroll
    const navbar = document.querySelector('.navbar');
    
    window.addEventListener('scroll', function() {
        if (window.scrollY > 50) {
            navbar.style.background = 'rgba(255, 255, 255, 0.98)';
            navbar.style.boxShadow = '0 2px 20px rgba(0, 0, 0, 0.1)';
        } else {
            navbar.style.background = 'rgba(255, 255, 255, 0.95)';
            navbar.style.boxShadow = 'none';
        }
    });
    
    // Intersection Observer for animations
    const observerOptions = {
        threshold: 0.1,
        rootMargin: '0px 0px -50px 0px'
    };
    
    const observer = new IntersectionObserver(function(entries) {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                entry.target.style.opacity = '1';
                entry.target.style.transform = 'translateY(0)';
            }
        });
    }, observerOptions);
    
    // Observe elements for animation
    const animatedElements = document.querySelectorAll('.feature-card, .usage-step, .install-card');
    animatedElements.forEach(el => {
        el.style.opacity = '0';
        el.style.transform = 'translateY(30px)';
        el.style.transition = 'opacity 0.6s ease, transform 0.6s ease';
        observer.observe(el);
    });
    
    // Terminal animation sequence
    function startTerminalAnimation() {
        const terminalLines = document.querySelectorAll('.terminal-line');
        let currentStep = 1;
        
        function showStep(step) {
            const line = document.querySelector(`[data-step="${step}"]`);
            if (line) {
                line.classList.remove('hidden');
                
                // If it's a typing text element, animate the typing
                const typingElement = line.querySelector('.typing-text');
                if (typingElement) {
                    const text = typingElement.getAttribute('data-text');
                    typingElement.textContent = '';
                    
                    let charIndex = 0;
                    const typeInterval = setInterval(() => {
                        if (charIndex < text.length) {
                            typingElement.textContent += text.charAt(charIndex);
                            charIndex++;
                        } else {
                            clearInterval(typeInterval);
                            // Remove cursor after typing is complete
                            setTimeout(() => {
                                typingElement.style.setProperty('--cursor-opacity', '0');
                            }, 500);
                            
                            // Move to next step
                            setTimeout(() => {
                                currentStep++;
                                if (currentStep <= 7) {
                                    showStep(currentStep);
                                }
                            }, 800);
                        }
                    }, 100);
                } else {
                    // For non-typing elements, just wait and move to next step
                    // Special handling for commit message (step 4) - shorter delay since it appears instantly
                    const delay = step === 4 ? 600 : 1000;
                    setTimeout(() => {
                        currentStep++;
                        if (currentStep <= 7) {
                            showStep(currentStep);
                        }
                    }, delay);
                }
            }
        }
        
        // Start the animation sequence
        setTimeout(() => {
            showStep(1);
        }, 1500);
    }
    
    // Start terminal animation when page loads
    startTerminalAnimation();
    
    // Add cursor movement animation after options appear
    setTimeout(() => {
        const optionLines = document.querySelectorAll('.option-line');
        let currentOption = 0;
        
        function moveCursor() {
            // Remove previous selection
            optionLines.forEach(line => {
                line.classList.remove('selected');
                line.innerHTML = line.innerHTML.replace('❯', ' ');
            });
            
            // Add selection to current option
            if (optionLines[currentOption]) {
                optionLines[currentOption].classList.add('selected');
                optionLines[currentOption].innerHTML = '❯' + optionLines[currentOption].innerHTML.substring(1);
            }
            
            // Move to next option
            currentOption = (currentOption + 1) % optionLines.length;
            
            // Continue animation
            setTimeout(moveCursor, 1500);
        }
        
        // Start cursor movement after options are fully visible
        setTimeout(moveCursor, 2000);
    }, 8000); // Wait for all steps to complete
    
    // Add hover effects to feature cards
    const featureCards = document.querySelectorAll('.feature-card');
    featureCards.forEach(card => {
        card.addEventListener('mouseenter', function() {
            this.style.transform = 'translateY(-8px) scale(1.02)';
        });
        
        card.addEventListener('mouseleave', function() {
            this.style.transform = 'translateY(0) scale(1)';
        });
    });
    
    // Add click effects to buttons
    const buttons = document.querySelectorAll('.btn');
    buttons.forEach(button => {
        button.addEventListener('click', function(e) {
            // Create ripple effect
            const ripple = document.createElement('span');
            const rect = this.getBoundingClientRect();
            const size = Math.max(rect.width, rect.height);
            const x = e.clientX - rect.left - size / 2;
            const y = e.clientY - rect.top - size / 2;
            
            ripple.style.width = ripple.style.height = size + 'px';
            ripple.style.left = x + 'px';
            ripple.style.top = y + 'px';
            ripple.classList.add('ripple');
            
            this.appendChild(ripple);
            
            setTimeout(() => {
                ripple.remove();
            }, 600);
        });
    });
});

// Add CSS for ripple effect
const style = document.createElement('style');
style.textContent = `
    .btn {
        position: relative;
        overflow: hidden;
    }
    
    .ripple {
        position: absolute;
        border-radius: 50%;
        background: rgba(255, 255, 255, 0.3);
        transform: scale(0);
        animation: ripple-animation 0.6s linear;
        pointer-events: none;
    }
    
    @keyframes ripple-animation {
        to {
            transform: scale(4);
            opacity: 0;
        }
    }
`;
document.head.appendChild(style); 