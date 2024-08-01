function scrollToBottom(id) {
    let scrollableDiv = document.getElementById(id);
    scrollableDiv.scrollTop = scrollableDiv.scrollHeight;
}; 
function scrollToBottom_chatbox(){
    scrollToBottom("chatbox");
}
function clear_input(name){ 
    var inputElement = document.querySelector('[name="'+name+'"]');
    inputElement.value = "";
}
 
    var simple_tab_handle = function(event) {
        if(event.target.className.split(" ").includes("simple_tab")){
            var simple_tab = document.getElementsByClassName("simple_tab");
            var simple_tab_content = document.getElementsByClassName("simple_tab_content");
            event.preventDefault();
            for (var i = 0; i < simple_tab_content.length; i++) {
                simple_tab_content[i].classList.remove('active');
            }
           
            for (var i = 0; i < simple_tab.length; i++) {
                simple_tab[i].classList.remove('active');
            }
            var href = event.target.getAttribute("id");
            event.target.classList.add("active");
            console.log(href+"-content");
            document.getElementById(href+"-content").classList.add("active");
            return false;
        }
    };

    document.addEventListener('click', simple_tab_handle, false);
 