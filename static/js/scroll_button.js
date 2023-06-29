  function CenteredScrollToElement(selector) {
    const element = document.querySelector(selector);
    if (element) {
      const offset = element.getBoundingClientRect().top;
      const scrollPosition = window.pageYOffset || document.documentElement.scrollTop;
      const targetPosition = offset + scrollPosition - (window.innerHeight / 2);
      window.scrollTo({ top: targetPosition, behavior: 'smooth' });
    }
  }

  function scrollToElement(selector) {
    const element = document.querySelector(selector);
    if (element) {
      element.scrollIntoView({ behavior: 'smooth' });
    }
  }