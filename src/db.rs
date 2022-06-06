async fn write_file(mut rx: mpsc::Receiver<Atom>) {
  println!("--- file 2 write start ---");
  let filepath = path::Path::new("test3.txt");
  let mut file = fs::File::create(&filepath).await.unwrap();
  while let Some(res) = rx.recv().await {
      // let n = file.write(&res.as_bytes()).await.unwrap();
      // println!("Wrote the first {} bytes of {}.", n, res);
      println!("Wrote the first");
  }
  println!("--- file 2 write end ---");
}