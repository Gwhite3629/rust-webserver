<!DOCTYPE html>
<?php
// requesting the value of the external variable "first_name" and filtering it.
$firstName = filter_input(INPUT_GET, "first_name", FILTER_SANITIZE_STRING);
?>
<html lang="en">
<head>
    <title>Output</title>
</head>
<body>
    <p>
        <?php echo "Hello, {$firstName}!"; /* printing the value */?>
    </p>
</body>
</html>