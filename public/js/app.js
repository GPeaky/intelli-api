"use strict";

// import * as te from 'tw-elements';
(function ($) {
  var currentPageUrl = window.location.href;
  var currentLink = currentPageUrl.split("/");
  var Href = currentLink[currentLink.length - 1];
  $('a[href="' + Href + '"]').addClass("active");
  var ParentUl = $("a.active").parent().parent();
  $(ParentUl).addClass("menu-open");
  var ParentClass = $("a.active").parent().parent().parent();
  $(ParentClass).addClass("active");
  function screenWidth() {
    if ($(window).width() < 1281) {
      $(".sidebar-wrapper").addClass("menu-hide");
      $("#menuCollapse").hide();
      $(".app-header").addClass("margin-0");
      $(".site-footer ").addClass("margin-0");
      $("#content_wrapper").addClass("margin-0");
      $(".sidebarCloseIcon").show();
      $("#sidebar_type").hide();
      $("#bodyOverlay").addClass("block");
    } else {
      $(".sidebar-wrapper").removeClass("menu-hide");
      $("#menuCollapse").show();
      $(".app-header").removeClass("margin-0");
      $(".site-footer").removeClass("margin-0");
      $("#content_wrapper").removeClass("margin-0");
      $(".sidebarCloseIcon").hide();
      $("#sidebar_type").show();
      $("#bodyOverlay").removeClass("block");
    }
  }
  screenWidth();
  $(window).resize(function () {
    screenWidth();
  });

  /*===================================
   Dark and light theme change
  =====================================*/
  var themes = [{
    name: "dark",
    "class": "dark",
    checked: false
  }, {
    name: "semiDark",
    "class": "semiDark",
    checked: false
  }, {
    name: "light",
    "class": "light",
    checked: false
  }];

  // Loop through themes and add event listener for changes
  themes.forEach(function (theme) {
    var radioBtn = $("#".concat(theme["class"]));
    radioBtn.prop("checked", theme.name === currentTheme);
    radioBtn.on("change", function () {
      if (this.checked) {
        currentTheme = theme.name;
        localStorage.theme = theme.name;
        location.reload();
      }
    });
  });

  // Theme Change by Header Button
  $("#themeMood").on("click", function () {
    if (currentTheme === "light") {
      currentTheme = "dark";
    } else {
      currentTheme = "light";
    }
    localStorage.theme = currentTheme;
    location.reload();
  });
  $("#grayScale").on("click", function () {
    if ($("html").hasClass("grayScale")) {
      $("html").removeClass("grayScale");
      localStorage.effect = "";
    } else {
      $("html").addClass("grayScale");
      localStorage.effect = "grayScale";
    }
  });

  /*===================================
   Layout Changer
  =====================================*/
  // Sidebar Type Local Storage save
  if (localStorage.sideBarType == "extend") {
    $(".app-wrapper").addClass(localStorage.sideBarType);
  } else if (localStorage.sideBarType == "collapsed") {
    $(".app-wrapper").removeClass("extend").addClass("collapsed");
    $("#menuCollapse input[type=checkbox]").prop("checked", true);
  }
  // Header Area Toggle switch
  $("#sidebar_type").on("click", function () {
    if ($(".app-wrapper").hasClass("collapsed")) {
      $(".app-wrapper").removeClass("collapsed").addClass("extend");
      $("#menuCollapse input[type=checkbox]").prop("checked", false);
      localStorage.sideBarType = "extend";
    } else {
      $(".app-wrapper").removeClass("extend").addClass("collapsed");
      $("#menuCollapse input[type=checkbox]").prop("checked", true);
      localStorage.sideBarType = "collapsed";
    }
  });

  // Settings Area Toggle Switch
  $("#menuCollapse input[type=checkbox]").on("click", function () {
    if ($("#menuCollapse input[type=checkbox]").is(":checked")) {
      $(".app-wrapper").removeClass("extend").addClass("collapsed");
      localStorage.sideBarType = "collapsed";
    } else {
      $(".app-wrapper").removeClass("collapsed").addClass("extend");
      localStorage.sideBarType = "extend";
    }
  });

  // Menu Hide and show toggle
  $("#menuHide input[type=checkbox]").on("click", function () {
    if ($("#menuHide input[type=checkbox]").is(":checked")) {
      $(".sidebar-wrapper").addClass("menu-hide");
      $("#menuCollapse").hide();
      $(".app-header").addClass("margin-0");
      $(".site-footer").addClass("margin-0");
      $("#content_wrapper").addClass("margin-0");
    } else {
      $(".sidebar-wrapper").removeClass("menu-hide");
      $("#menuCollapse").show();
      $(".app-header").removeClass("margin-0");
      $(".site-footer").removeClass("margin-0");
      $("#content_wrapper").removeClass("margin-0");
    }
  });

  // Content layout toggle
  if (localStorage.contentLayout == "container") {
    $("#page_layout").addClass(localStorage.contentLayout);
    $("#boxed").prop("checked", true);
  } else {
    $("#page_layout").addClass("container-fluid");
    $("#fullWidth").prop("checked", true);
  }

  // Content layout Changing options
  $("#fullWidth").on("change", function () {
    $("#page_layout").removeClass("container").addClass("container-fluid");
    localStorage.contentLayout = "container-fluid";
  });
  $("#boxed").on("change", function () {
    $("#page_layout").removeClass("container-fluid").addClass("container");
    localStorage.contentLayout = "container";
  });

  // Menu Layout toggle
  if (localStorage.menuLayout == "horizontalMenu") {
    // $(".app-wrapper").addClass(localStorage.menuLayout);
    $("#horizontal_menu").prop("checked", true);
  } else {
    // $(".app-wrapper").removeClass("horizontalMenu");
    $("#vertical_menu").prop("checked", true);
  }

  // Menu Layout Changing options
  $("#vertical_menu").on("change", function () {
    $(".app-wrapper").removeClass("horizontalMenu");
    localStorage.menuLayout = "";
  });
  $("#horizontal_menu").on("change", function () {
    $(".app-wrapper").addClass("horizontalMenu");
    localStorage.menuLayout = "horizontalMenu";
  });

  // Header Area styles

  // Check local storage and set Header Style
  if (localStorage.navbar == "floating") {
    $("#app_header").addClass(localStorage.navbar);
    $("#nav_" + localStorage.navbar).prop("checked", true);
  } else if (localStorage.navbar == "sticky top-0") {
    $("#app_header").addClass(localStorage.navbar);
    $("#nav_sticky").prop("checked", true);
  } else if (localStorage.navbar == "hidden") {
    $("#app_header").addClass(localStorage.navbar);
    $("#nav_" + localStorage.navbar).prop("checked", true);
  } else {
    $("#app_header").addClass("static");
    $("#nav_static").prop("checked", true);
  }
  // Header Changing options
  $("#nav_floating").on("change", function () {
    $("#app_header").removeClass("sticky top-0").removeClass("hidden").removeClass("static").addClass("floating");
    localStorage.navbar = "floating";
  });
  $("#nav_sticky").on("change", function () {
    $("#app_header").removeClass("floating").removeClass("hidden").removeClass("static").addClass("sticky top-0");
    localStorage.navbar = "sticky top-0";
  });
  $("#nav_static").on("change", function () {
    $("#app_header").removeClass("floating").removeClass("hidden").removeClass("sticky top-0").addClass("static");
    localStorage.navbar = "static";
  });
  $("#nav_hidden").on("change", function () {
    $("#app_header").removeClass("floating").removeClass("static").removeClass("sticky top-0").addClass("hidden");
    localStorage.navbar = "hidden";
  });

  // Footer Area
  // Check local storage and set Footer Style
  if (localStorage.footer == "sticky bottom-0") {
    $("#footer").addClass(localStorage.footer);
    $("#footer_sticky").prop("checked", true);
  } else if (localStorage.footer == "hidden") {
    $("#footer").addClass(localStorage.footer);
    $("#footer_hidden").prop("checked", true);
  } else {
    $("#footer").addClass("static");
    $("#footer_static").prop("checked", true);
  }
  // Footer Changing options
  $("#footer_static").on("change", function () {
    $("#footer").removeClass("sticky bottom-0").removeClass("hidden").addClass("static");
    localStorage.footer = "static";
  });
  $("#footer_sticky").on("change", function () {
    $("#footer").removeClass("static").removeClass("hidden").addClass("sticky bottom-0");
    localStorage.footer = "sticky bottom-0";
  });
  $("#footer_hidden").on("change", function () {
    $("#footer").removeClass("sticky bottom-0").removeClass("static").addClass("hidden");
    localStorage.footer = "hidden";
  });

  // RTL and LTR
  // Direction Type Local Storage
  if (localStorage.dir == "rtl") {
    $("#rtl_ltr input[type=checkbox]").prop("checked", true);
    $('#offcanvas').removeClass('offcanvas-end');
    $('#offcanvas').addClass('offcanvas-start');
  }

  // Change Direction
  $("#rtl_ltr input[type=checkbox]").on("click", function () {
    if ($("#rtl_ltr input[type=checkbox]").is(":checked")) {
      $("html").attr("dir", "rtl");
      localStorage.dir = "rtl";
      location.reload();
    } else {
      $("html").attr("dir", "ltr");
      localStorage.dir = "ltr";
      location.reload();
    }
  });

  /* =============================
  Small Device Buttons function
  ===============================*/
  $(".smallDeviceMenuController").on("click", function () {
    $(".sidebar-wrapper").toggleClass("menu-hide");
    $("#bodyOverlay").removeClass("hidden");
    $("body").addClass("overflow-hidden");
  });
  $(".sidebarCloseIcon, #bodyOverlay").on("click", function () {
    $(".sidebar-wrapper").toggleClass("menu-hide");
    $("#bodyOverlay").addClass("hidden");
    $("body").removeClass("overflow-hidden");
  });

  // Password Show Hide Toggle
  $("#toggleIcon").on("click", function () {
    var x = $(".passwordfield").attr("type");
    if (x === "password") {
      $(".passwordfield").prop("type", "text");
      $("#hidePassword").hide();
      $("#showPassword").show();
    } else {
      $(".passwordfield").prop("type", "password");
      $("#showPassword").hide();
      $("#hidePassword").show();
    }
  });

  // Getting the Current Year
  $("#thisYear").text(new Date().getFullYear());

  // Progress bar
  $(".progress-bar").animate({
    width: "40%"
  }, 2500);
  $(".progress-bar2").animate({
    width: "50%"
  }, 2500);
  $(".progress-bar3").animate({
    width: "60%"
  }, 2500);
  $(".progress-bar4").animate({
    width: "75%"
  }, 2500);
  $(".progress-bar5").animate({
    width: "95%"
  }, 2500);
  $(".progress-bar6").animate({
    width: "25%"
  }, 2500);

  /*===================================
   Plugin initialization
  =====================================*/
  // Sidebar Menu
  $.sidebarMenu($(".sidebar-menu"));

  // Simple Bar
  new SimpleBar($("#sidebar_menus, #scrollModal")[0]);

  // Basic Carousel
  $(".basic-carousel").owlCarousel({
    loop: true,
    nav: true,
    items: 1,
    lazyLoad: true,
    navText: ['<iconify-icon icon="ic:round-arrow-back-ios"></iconify-icon>', '<iconify-icon icon="ic:round-arrow-forward-ios"></iconify-icon>']
  });

  // Carousel Interval
  $(".carousel-interval").owlCarousel({
    autoplay: true,
    autoplayTimeout: 3000,
    autoplayHoverPause: true,
    lazyLoad: true,
    loop: true,
    nav: true,
    items: 1,
    navText: ['<iconify-icon icon="ic:round-arrow-back-ios"></iconify-icon>', '<iconify-icon icon="ic:round-arrow-forward-ios"></iconify-icon>']
  });

  //Carousel Crossfade
  $(".carousel-crossfade").owlCarousel({
    autoplay: true,
    autoplayTimeout: 3000,
    autoplayHoverPause: true,
    lazyLoad: true,
    loop: true,
    nav: true,
    items: 1,
    animateIn: "fadeIn",
    animateOut: "fadeOut",
    navText: ['<iconify-icon icon="ic:round-arrow-back-ios"></iconify-icon>', '<iconify-icon icon="ic:round-arrow-forward-ios"></iconify-icon>']
  });

  // Video Player
  var player = new Plyr("#player", {
    controls: ["play-large", "current-time", "progress", "mute", "volume", "pip", "fullscreen", "settings"]
  });

  // Tooltip and Popover
  tippy(".onTop", {
    content: "Tooltip On Top!",
    placement: "top"
  });
  tippy(".onRight", {
    content: "Tooltip On Right!",
    placement: "right"
  });
  tippy(".onBottom", {
    content: "Tooltip On Bottom!",
    placement: "bottom"
  });
  tippy(".onLeft", {
    content: "Tooltip On Left!",
    placement: "left"
  });

  // ToolTip Animations
  tippy(".scale", {
    placement: "top",
    animation: "scale"
  });
  tippy(".shift-Away", {
    placement: "top",
    animation: "shift-away"
  });
  tippy(".shift-Toward", {
    placement: "top",
    animation: "shift-toward"
  });
  tippy(".perspective", {
    placement: "top",
    animation: "perspective"
  });
  tippy(".onClickTooltip", {
    placement: "top",
    animation: "shift-away",
    trigger: "click"
  });

  // Form Validation
  $("#loginForm").on("submit", function (event) {
    event.preventDefault(); // prevent form submission

    // get values of email and password fields
    var name = $("#name").val();
    var email = $("#email").val();

    // validate email and password
    if (name == "" && !isValidEmail(email)) {
      $("#nameErrorMsg").text("Please enter your name.").show();
      $("#emailErrorMsg").text("Please enter a valid email address.").show();
    } else if (name == "") {
      $("#nameErrorMsg").text("Please enter your name.").show();
      $("#emailErrorMsg").text("Please enter a valid email address.").hide();
    } else if (!isValidEmail(email)) {
      $("#nameErrorMsg").text("Please enter your name.").hide();
      $("#emailErrorMsg").text("Please enter a valid email address.").show();
    } else {
      // submit form if email and password are valid
      $("#nameErrorMsg").text("Please enter your name.").hide();
      $("#emailErrorMsg").text("Please enter a valid email address.").hide();
      $("#login-form").submit();
      console.log(name, email);
    }
  });
  $("#passIcon").on("click", function () {
    $("#passIcon iconify-icon").toggle();
  });
  $("#ConfirmpassIcon").on("click", function () {
    $("#ConfirmpassIcon iconify-icon").toggle();
  });

  // function to validate email
  function isValidEmail(email) {
    // use regular expression to validate email
    var emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return emailRegex.test(email);
  }

  // function to validate password
  function isValidPassword(password) {
    // password must be at least 8 characters long and include at least one uppercase letter, one lowercase letter, and one number
    var passwordRegex = /^(?=.*\d)(?=.*[a-z])(?=.*[A-Z])[a-zA-Z\d]{8,}$/;
    return passwordRegex.test(password);
  }
  $("#tooltipValidation").validate({
    errorElement: "span",
    rules: {
      tooltip_name: {
        required: true
      },
      tooltip_email: {
        required: true,
        email: true
      }
    },
    messages: {
      tooltip_name: "Please enter your name",
      tooltip_email: {
        required: "Enter your email",
        email: "Enter a valid email"
      }
    }
  });
  $("#multipleValidation").validate({
    errorElement: "span",
    rules: {
      name: {
        required: true
      },
      email: {
        required: true,
        email: true
      },
      password: {
        required: true,
        minlength: 6
      },
      confirm_password: {
        required: true,
        minlength: 6,
        equalTo: "#password"
      }
    },
    messages: {
      name: "Please enter your name",
      email: {
        required: "Enter your email",
        email: "Enter a valid email"
      },
      password: {
        required: "Enter your password",
        minlength: "Password should contain minimum 6 character"
      },
      confirm_password: {
        required: "Enter your password",
        minlength: "Password should contain minimum 6 character",
        equalTo: "Did not match the password"
      }
    }
  });
  $("#typeValidation").validate({
    errorElement: "span",
    rules: {
      name: {
        required: true
      },
      number: {
        required: true,
        number: true
      },
      Password: {
        required: true,
        minlength: 8
      },
      rangeType: {
        required: true,
        range: [0, 50]
      },
      specifiedLength: {
        required: true,
        minlength: 3
      },
      alphabeticCharacter: {
        required: true,
        number: false
      },
      url: {
        required: true,
        url: true
      },
      textMsg: {
        required: true
      }
    },
    messages: {
      name: "Please enter your name",
      number: {
        required: "Please enter number"
      },
      Password: {
        required: "Enter your password",
        minlength: "Password should contain minimum 8 character"
      },
      specifiedLength: {
        minlength: "Should contain minimum 3 character"
      },
      url: {
        url: "Invalid URL"
      }
    }
  });

  // data table validation
  $("#data-table, .data-table").DataTable({
    dom: "<'grid grid-cols-12 gap-5 px-6 mt-6'<'col-span-4'l><'col-span-8 flex justify-end'f><'#pagination.flex items-center'>><'min-w-full't><'flex justify-end items-center'p>",
    paging: true,
    ordering: true,
    info: false,
    searching: true,
    lengthChange: true,
    lengthMenu: [10, 25, 50, 100],
    language: {
      lengthMenu: "Show _MENU_ entries",
      paginate: {
        previous: "<iconify-icon icon=\"ic:round-keyboard-arrow-left\"></iconify-icon>",
        next: "<iconify-icon icon=\"ic:round-keyboard-arrow-right\"></iconify-icon>"
      },
      search: "Search:"
    }
  });

  // flatpickr
  $(".flatpickr").flatpickr({
    dateFormat: "Y-m-d",
    defaultDate: "today"
  });
  if (document.getElementById("map") || document.getElementById("map2")) {
    // map active
    var position = [47.31322, -1.319482];
    var map = L.map("map").setView(position, 8);
    L.tileLayer("https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png", {
      attribution: '&copy; <a href="https://www.openstreetmap.org/">OpenStreetMap</a> contributors',
      maxZoom: 18
    }).addTo(map);
    L.marker(position).addTo(map).bindPopup('<div class="">A pretty CSS3 popup. <br /> Easily customizable.</div>').openPopup();

    // marker map
    var circleRadius = 4500;
    var polygonCoords = [[47.2263299, -1.6222], [47.21024000000001, -1.6270065], [47.1969447, -1.6136169], [47.18527929999999, -1.6143036], [47.1794457, -1.6098404], [47.1775788, -1.5985107], [47.1676598, -1.5753365], [47.1593731, -1.5521622], [47.1593731, -1.5319061], [47.1722111, -1.5143967], [47.1960115, -1.4841843], [47.2095404, -1.4848709], [47.2291277, -1.4683914], [47.2533687, -1.5116501], [47.2577961, -1.5531921], [47.26828069, -1.5621185], [47.2657179, -1.589241], [47.2589612, -1.6204834], [47.237287, -1.6266632], [47.2263299, -1.6222]];
    var map2 = L.map("map2").setView(position, 10);
    L.tileLayer("https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png", {
      attribution: '&copy; <a href="https://www.openstreetmap.org/">OpenStreetMap</a> contributors'
    }).addTo(map2);
    L.marker(position).addTo(map2);
    L.circle(position, {
      radius: circleRadius
    }).addTo(map2);
    L.polygon(polygonCoords).addTo(map2);
  }

  // geo map

  // Using Options Array Checkbox
  $('input[name="arrayCheckbox[]"]').on("click", function () {
    var selectedItem = $('input[name="arrayCheckbox[]"]:checked').map(function () {
      return $(this).val();
    }).get();
    $("#selectedItems").text(selectedItem.join(", "));
  });

  // Using Options Array Radio
  $('input[name="arrayRadio[]"]').on("click", function () {
    var radioSelectedItem = $('input[name="arrayRadio[]"]:checked').map(function () {
      return $(this).val();
    }).get();
    $("#radioSelectedItems").text(radioSelectedItem.join(", "));
  });

  // Input Validation with cleave js
  var cleaveConfigs = [{
    element: "#creditCard",
    options: {
      creditCard: true
    }
  }, {
    element: "#phone",
    options: {
      prefix: "+88 ",
      delimiter: "-",
      blocks: [8, 7],
      number: true
    }
  }, {
    element: "#date",
    options: {
      date: true,
      delimiter: "/",
      datePattern: ["Y", "m", "d"]
    }
  }, {
    element: "#time",
    options: {
      time: true,
      timePattern: ["h", "m", "s"]
    }
  }, {
    element: "#numeralFormatting",
    options: {
      numeral: true,
      numeralThousandsGroupStyle: "thousand"
    }
  }, {
    element: "#blocks",
    options: {
      blocks: [4, 3, 3, 4],
      uppercase: true
    }
  }, {
    element: "#delimiters",
    options: {
      delimiter: ".",
      blocks: [3, 3, 3],
      uppercase: true
    }
  }, {
    element: "#customdelimiters",
    options: {
      delimiters: [".", "/", "-"],
      blocks: [3, 3, 3, 2],
      uppercase: true
    }
  }, {
    element: "#prefix",
    options: {
      prefix: "+88 ",
      delimiter: "-",
      blocks: [6, 4, 4, 4],
      uppercase: true
    }
  }];
  cleaveConfigs.forEach(function (item) {
    var element = document.getElementById(item.element);
    if (element) {
      new Cleave(item.element, item.options);
    }
  });

  // Form Select Area
  $(".select2").select2({
    placeholder: "Select an Option"
  });
  $("#limitedSelect").select2({
    placeholder: "Select an Option",
    maximumSelectionLength: 2
  });
  $(".filegroup input").on("change", function () {
    var count = $(this).get(0).files.length;
    var message = count + " file" + (count === 1 ? "" : "s") + " selected";
    $("#placeholder").text(message);
  });
  $(".filePreview input").on("change", function () {
    var files = $(this).get(0).files;
    var preview = $(".filePreview #file-preview");
    var name = $(".filePreview #placeholder");
    preview.empty();
    name.empty();
    if (files) {
      for (var i = 0; i < files.length; i++) {
        var file = files[i];
        var reader = new FileReader();
        reader.onload = function () {
          var img = $("<img>").attr("src", reader.result);
          preview.append(img);
        };
        reader.readAsDataURL(file);
        var span = $("<span>").text(file.name);
        name.append(span);
      }
    }
  });
  $(".multiFilePreview input").on("change", function () {
    $(".multiFilePreview #file-preview").empty(); // clear any existing previews
    var files = $(this)[0].files;
    var count = files.length;
    $(".multiFilePreview #placeholder").text(count + " file(s) selected");
    for (var i = 0; i < files.length; i++) {
      var file = files[i];
      var reader = new FileReader();
      reader.onload = function (event) {
        var img = $("<img>").attr("src", event.target.result);
        $(".multiFilePreview #file-preview").append(img);
      };
      reader.readAsDataURL(file);
    }
  });
  Dropzone.autoDiscover = false;
  $("#myUploader").dropzone({
    url: "/",
    dictDefaultMessage: "",
    addRemoveLinks: true
  });

  // Flatpickr
  $("#disabled-range-picker").flatpickr({
    mode: "range",
    minDate: "today",
    dateFormat: "Y-m-d",
    disable: [function (date) {
      // disable every multiple of 8
      return !(date.getDate() % 8);
    }]
  });
  $(".flatpickr.time").flatpickr({
    enableTime: true,
    noCalendar: true,
    dateFormat: "H:i",
    time_24hr: true
  });
  $("#humanFriendly_picker").flatpickr({
    altInput: true,
    altFormat: "F j, Y",
    dateFormat: "Y-m-d"
  });
  $("#inline_picker").flatpickr({
    inline: true,
    altInput: true,
    altFormat: "F j, Y",
    dateFormat: "Y-m-d"
  });

  // Get all checkboxes and list items
  var checkboxes = document.querySelectorAll('input[name="tasklist"]');

  // Add event listener to each checkbox
  checkboxes.forEach(function (checkbox) {
    checkbox.addEventListener("change", function () {
      // Get parent list item
      var listItem = checkbox.closest("li");

      // Toggle class based on checkbox state
      if (checkbox.checked) {
        listItem.classList.add("completed");
      } else {
        listItem.classList.remove("completed");
      }
    });
  });
  var swiper = new Swiper(".card-slider", {
    effect: "cards",
    grabCursor: true
  });

  // Dragula for Kanban
  dragula([document.getElementById("todo"), document.getElementById("progress"), document.getElementById("done")]);

  // Step From
  $("#example-basic").steps({
    headerTag: "h3",
    bodyTag: "section",
    transitionEffect: "slideLeft",
    autoFocus: true
  });

  // Quill Editor For Compose Email Modal

  var $quill = $("#editor-container");
  $quill.each(function () {
    var quill = new Quill(this, {
      modules: {
        toolbar: [[{
          header: [1, 2, 3, false]
        }], ["bold", "italic", "underline"], ["image", "code-block"]]
      },
      placeholder: "Your Email",
      theme: "snow"
    });
  });
  // vector map init
  $("#world-map").vectorMap({
    map: "world_mill_en",
    normalizeFunction: "polynomial",
    hoverOpacity: 0.7,
    hoverColor: false,
    regionStyle: {
      initial: {
        fill: "#8092FF"
      },
      hover: {
        fill: "#4669fa",
        "fill-opacity": 1
      }
    },
    backgroundColor: "transparent"
  });
  $("#dashcode-mini-calendar").zabuto_calendar({
    header_format: "[year] - [month]",
    week_starts: "sunday",
    show_days: true,
    today_markup: '<span class="badge bg-slate-900 dark:bg-slate-700 text-white dark:text-slate-300">[day]</span>',
    navigation_markup: {
      prev: '<iconify-icon icon="heroicons-outline:chevron-left"></iconify-icon>',
      next: '<iconify-icon icon="heroicons-outline:chevron-right"></iconify-icon>'
    }
  });
})(jQuery);