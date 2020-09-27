function store(val) {
  a = val;
}
{
  print($0, a);
  b = store($0);
}
